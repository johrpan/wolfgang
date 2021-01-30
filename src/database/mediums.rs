use super::schema::{mediums, track_sets, tracks};
use super::{get_recording, update_recording};
use super::{DbConn, Recording, User};
use crate::error::ServerError;
use anyhow::{anyhow, Error, Result};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A medium containing multiple recordings.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Medium {
    /// An unique ID for the medium.
    pub id: String,

    /// The human identifier for the medium.
    pub name: String,

    /// If applicable, the MusicBrainz DiscID.
    pub discid: Option<String>,

    /// The tracks of the medium, grouped by recording.
    pub tracks: Vec<TrackSet>,
}

/// A set of tracks of one recording within a medium.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TrackSet {
    /// The recording to which the tracks belong.
    pub recording: Recording,

    /// The actual tracks.
    pub tracks: Vec<Track>,
}

/// A track within a recording on a medium.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    /// The work parts that are played on this track. They are indices to the
    /// work parts of the work that is associated with the recording.
    pub work_parts: Vec<usize>,
}

/// Table data for a [`Medium`].
#[derive(Insertable, Queryable, Debug, Clone)]
#[table_name = "mediums"]
struct MediumRow {
    pub id: String,
    pub name: String,
    pub discid: Option<String>,
    pub created_by: String,
}

/// Table data for a [`TrackSet`].
#[derive(Insertable, Queryable, Debug, Clone)]
#[table_name = "track_sets"]
struct TrackSetRow {
    pub id: i64,
    pub medium: String,
    pub index: i32,
    pub recording: String,
}

/// Table data for a [`Track`].
#[derive(Insertable, Queryable, Debug, Clone)]
#[table_name = "tracks"]
struct TrackRow {
    pub id: i64,
    pub track_set: i64,
    pub index: i32,
    pub work_parts: String,
}

/// Update an existing medium or insert a new one. This will only work, if the provided user is
/// allowed to do that.
pub fn update_medium(conn: &DbConn, medium: &Medium, user: &User) -> Result<()> {
    conn.transaction::<(), Error, _>(|| {
        let old_row = get_medium_row(conn, &medium.id)?;

        let allowed = match old_row {
            Some(row) => user.may_edit(&row.created_by),
            None => user.may_create(),
        };

        if allowed {
            let id = &medium.id;

            // This will also delete the track sets and tracks.

            diesel::delete(mediums::table)
                .filter(mediums::id.eq(id))
                .execute(conn)?;

            // Add the actual medium first.

            let row = MediumRow {
                id: id.clone(),
                name: medium.name.clone(),
                discid: medium.discid.clone(),
                created_by: user.username.clone(),
            };

            diesel::insert_into(mediums::table)
                .values(row)
                .execute(conn)?;

            // Add the track sets.

            for (index, track_set) in medium.tracks.iter().enumerate() {
                // Add the associated recording, if it doesn't exist.

                if get_recording(conn, &track_set.recording.id)?.is_none() {
                    update_recording(conn, &track_set.recording, user)?;
                }

                // Add the track set itself.

                let track_set_id = rand::random();

                let track_set_row = TrackSetRow {
                    id: track_set_id,
                    medium: id.clone(),
                    index: index as i32,
                    recording: track_set.recording.id.clone(),
                };

                diesel::insert_into(track_sets::table)
                    .values(track_set_row)
                    .execute(conn)?;

                // Add the tracks within the track set.

                for (index, track) in track_set.tracks.iter().enumerate() {
                    let work_parts = track
                        .work_parts
                        .iter()
                        .map(|part_index| part_index.to_string())
                        .collect::<Vec<String>>()
                        .join(",");

                    let track_row = TrackRow {
                        id: rand::random(),
                        track_set: track_set_id,
                        index: index as i32,
                        work_parts,
                    };

                    diesel::insert_into(tracks::table)
                        .values(track_row)
                        .execute(conn)?;
                }
            }

            Ok(())
        } else {
            Err(Error::new(ServerError::Forbidden))
        }
    })?;

    Ok(())
}

/// Get an existing medium and all available information from related tables.
pub fn get_medium(conn: &DbConn, id: &str) -> Result<Option<Medium>> {
    let medium = match get_medium_row(conn, id)? {
        Some(row) => Some(get_medium_data(conn, row)?),
        None => None,
    };

    Ok(medium)
}

/// Get mediums that contain a specific recording.
pub fn get_mediums_for_recording(conn: &DbConn, recording_id: &str) -> Result<Vec<Medium>> {
    let mut mediums: Vec<Medium> = Vec::new();

    let rows = mediums::table
        .inner_join(track_sets::table.on(track_sets::medium.eq(mediums::id)))
        .filter(track_sets::recording.eq(recording_id))
        .select(mediums::table::all_columns())
        .load::<MediumRow>(conn)?;

    for row in rows {
        let medium = get_medium_data(conn, row)?;
        mediums.push(medium);
    }

    Ok(mediums)
}

/// Get mediums that have a specific DiscID.
pub fn get_mediums_by_discid(conn: &DbConn, discid: &str) -> Result<Vec<Medium>> {
    let mut mediums: Vec<Medium> = Vec::new();

    let rows = mediums::table
        .filter(mediums::discid.nullable().eq(discid))
        .load::<MediumRow>(conn)?;

    for row in rows {
        let medium = get_medium_data(conn, row)?;
        mediums.push(medium);
    }

    Ok(mediums)
}

/// Get an existing medium row.
fn get_medium_row(conn: &DbConn, id: &str) -> Result<Option<MediumRow>> {
    Ok(mediums::table
        .filter(mediums::id.eq(id))
        .load::<MediumRow>(conn)?
        .into_iter()
        .next())
}

/// Retrieve all available information on a medium from related tables.
fn get_medium_data(conn: &DbConn, row: MediumRow) -> Result<Medium> {
    let track_set_rows = track_sets::table
        .filter(track_sets::medium.eq(&row.id))
        .order_by(track_sets::index)
        .load::<TrackSetRow>(conn)?;

    let mut track_sets = Vec::new();

    for track_set_row in track_set_rows {
        let track_set = get_track_set_from_row(conn, track_set_row)?;
        track_sets.push(track_set);
    }

    let medium = Medium {
        id: row.id,
        name: row.name,
        discid: row.discid,
        tracks: track_sets,
    };

    Ok(medium)
}

/// Convert a track set row from the database to an actual track set.
fn get_track_set_from_row(conn: &DbConn, row: TrackSetRow) -> Result<TrackSet> {
    let recording_id = row.recording;

    let recording = get_recording(conn, &recording_id)?
        .ok_or_else(|| anyhow!("No recording with ID: {}", recording_id))?;

    let track_rows = tracks::table
        .filter(tracks::track_set.eq(row.id))
        .order_by(tracks::index)
        .load::<TrackRow>(conn)?;

    let mut tracks = Vec::new();

    for track_row in track_rows {
        let work_parts = track_row
            .work_parts
            .split(',')
            .map(|part_index| Ok(str::parse(part_index)?))
            .collect::<Result<Vec<usize>>>()?;

        let track = Track {
            work_parts,
        };

        tracks.push(track);
    }

    let track_set = TrackSet { recording, tracks };

    Ok(track_set)
}

/// Delete an existing medium. This will fail if there are still references to this
/// medium from other tables that are not directly part of the recording data. Also, the
/// provided user has to be allowed to delete the recording.
pub fn delete_medium(conn: &DbConn, id: &str, user: &User) -> Result<()> {
    if user.may_delete() {
        diesel::delete(mediums::table.filter(mediums::id.eq(id))).execute(conn)?;
        Ok(())
    } else {
        Err(Error::new(ServerError::Forbidden))
    }
}

