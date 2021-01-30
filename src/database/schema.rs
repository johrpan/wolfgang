table! {
    ensembles (id) {
        id -> Text,
        name -> Text,
        created_by -> Text,
    }
}

table! {
    instrumentations (id) {
        id -> Int8,
        work -> Text,
        instrument -> Text,
    }
}

table! {
    instruments (id) {
        id -> Text,
        name -> Text,
        created_by -> Text,
    }
}

table! {
    mediums (id) {
        id -> Text,
        name -> Text,
        discid -> Nullable<Text>,
        created_by -> Text,
    }
}

table! {
    performances (id) {
        id -> Int8,
        recording -> Text,
        person -> Nullable<Text>,
        ensemble -> Nullable<Text>,
        role -> Nullable<Text>,
    }
}

table! {
    persons (id) {
        id -> Text,
        first_name -> Text,
        last_name -> Text,
        created_by -> Text,
    }
}

table! {
    recordings (id) {
        id -> Text,
        work -> Text,
        comment -> Text,
        created_by -> Text,
    }
}

table! {
    track_sets (id) {
        id -> Int8,
        medium -> Text,
        index -> Int4,
        recording -> Text,
    }
}

table! {
    tracks (id) {
        id -> Int8,
        track_set -> Int8,
        index -> Int4,
        work_parts -> Text,
    }
}

table! {
    users (username) {
        username -> Text,
        password_hash -> Text,
        email -> Nullable<Text>,
        is_admin -> Bool,
        is_editor -> Bool,
        is_banned -> Bool,
    }
}

table! {
    work_parts (id) {
        id -> Int8,
        work -> Text,
        part_index -> Int8,
        title -> Text,
        composer -> Nullable<Text>,
    }
}

table! {
    work_sections (id) {
        id -> Int8,
        work -> Text,
        title -> Text,
        before_index -> Int8,
    }
}

table! {
    works (id) {
        id -> Text,
        composer -> Text,
        title -> Text,
        created_by -> Text,
    }
}

joinable!(ensembles -> users (created_by));
joinable!(instrumentations -> instruments (instrument));
joinable!(instrumentations -> works (work));
joinable!(instruments -> users (created_by));
joinable!(mediums -> users (created_by));
joinable!(performances -> ensembles (ensemble));
joinable!(performances -> instruments (role));
joinable!(performances -> persons (person));
joinable!(performances -> recordings (recording));
joinable!(persons -> users (created_by));
joinable!(recordings -> users (created_by));
joinable!(recordings -> works (work));
joinable!(track_sets -> mediums (medium));
joinable!(track_sets -> recordings (recording));
joinable!(tracks -> track_sets (track_set));
joinable!(work_parts -> persons (composer));
joinable!(work_parts -> works (work));
joinable!(work_sections -> works (work));
joinable!(works -> persons (composer));
joinable!(works -> users (created_by));

allow_tables_to_appear_in_same_query!(
    ensembles,
    instrumentations,
    instruments,
    mediums,
    performances,
    persons,
    recordings,
    track_sets,
    tracks,
    users,
    work_parts,
    work_sections,
    works,
);
