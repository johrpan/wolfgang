use super::authenticate;
use crate::database;
use crate::database::{DbPool, Medium};
use crate::error::ServerError;
use actix_web::{delete, get, post, web, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;

/// Get an existing medium by ID.
#[get("/mediums/{id}")]
pub async fn get_medium(
    db: web::Data<DbPool>,
    id: web::Path<String>,
) -> Result<HttpResponse, ServerError> {
    let data = web::block(move || {
        let conn = db.into_inner().get()?;
        database::get_medium(&conn, &id.into_inner())?.ok_or(ServerError::NotFound)
    })
    .await?;

    Ok(HttpResponse::Ok().json(data))
}

/// Add a new medium or update an existing one. The user must be authorized to do that.
#[post("/mediums")]
pub async fn update_medium(
    auth: BearerAuth,
    db: web::Data<DbPool>,
    data: web::Json<Medium>,
) -> Result<HttpResponse, ServerError> {
    web::block(move || {
        let conn = db.into_inner().get()?;
        let user = authenticate(&conn, auth.token()).or(Err(ServerError::Unauthorized))?;

        database::update_medium(&conn, &data.into_inner(), &user)?;

        Ok(())
    })
    .await?;

    Ok(HttpResponse::Ok().finish())
}

#[get("/recordings/{id}/mediums")]
pub async fn get_mediums_for_recording(
    db: web::Data<DbPool>,
    recording_id: web::Path<String>,
) -> Result<HttpResponse, ServerError> {
    let data = web::block(move || {
        let conn = db.into_inner().get()?;
        Ok(database::get_mediums_for_recording(&conn, &recording_id.into_inner())?)
    })
    .await?;

    Ok(HttpResponse::Ok().json(data))
}

#[get("/discids/{id}/mediums")]
pub async fn get_mediums_by_discid(
    db: web::Data<DbPool>,
    discid: web::Path<String>,
) -> Result<HttpResponse, ServerError> {
    let data = web::block(move || {
        let conn = db.into_inner().get()?;
        Ok(database::get_mediums_by_discid(&conn, &discid.into_inner())?)
    })
    .await?;

    Ok(HttpResponse::Ok().json(data))
}

#[delete("/mediums/{id}")]
pub async fn delete_medium(
    auth: BearerAuth,
    db: web::Data<DbPool>,
    id: web::Path<String>,
) -> Result<HttpResponse, ServerError> {
    web::block(move || {
        let conn = db.into_inner().get()?;
        let user = authenticate(&conn, auth.token()).or(Err(ServerError::Unauthorized))?;

        database::delete_medium(&conn, &id.into_inner(), &user)?;

        Ok(())
    })
    .await?;

    Ok(HttpResponse::Ok().finish())
}
