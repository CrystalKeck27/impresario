// @generated automatically by Diesel CLI.

diesel::table! {
    acoustid_fingerprints (impresario_id) {
        impresario_id -> Uuid,
        fingerprint -> Bytea,
        duration_seconds -> Int4,
    }
}

diesel::table! {
    acoustid_matches (impresario_id, acoustid_id) {
        impresario_id -> Uuid,
        acoustid_id -> Uuid,
        score -> Float4,
    }
}

diesel::table! {
    impresario_files (id) {
        id -> Uuid,
        file_path -> Text,
    }
}

diesel::table! {
    musicbrainz_recordings (impresario_id) {
        impresario_id -> Uuid,
        recording_id -> Uuid,
    }
}

diesel::table! {
    musicbrainz_tracks (impresario_id) {
        impresario_id -> Uuid,
        track_id -> Uuid,
    }
}

diesel::joinable!(acoustid_fingerprints -> impresario_files (impresario_id));
diesel::joinable!(acoustid_matches -> impresario_files (impresario_id));
diesel::joinable!(musicbrainz_recordings -> impresario_files (impresario_id));
diesel::joinable!(musicbrainz_tracks -> impresario_files (impresario_id));

diesel::allow_tables_to_appear_in_same_query!(
    acoustid_fingerprints,
    acoustid_matches,
    impresario_files,
    musicbrainz_recordings,
    musicbrainz_tracks,
);
