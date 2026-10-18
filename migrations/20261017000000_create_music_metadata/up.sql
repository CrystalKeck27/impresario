-- Mirrors scripts/create-tables.sql.
CREATE TABLE IF NOT EXISTS impresario_files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    file_path TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS acoustid_fingerprints (
    impresario_id UUID PRIMARY KEY
        REFERENCES impresario_files (id)
        ON DELETE CASCADE,
    fingerprint BYTEA NOT NULL,
    duration_seconds INTEGER NOT NULL
        CHECK (duration_seconds >= 0)
);

CREATE TABLE IF NOT EXISTS musicbrainz_recordings (
    impresario_id UUID PRIMARY KEY
        REFERENCES impresario_files (id)
        ON DELETE CASCADE,
    recording_id UUID NOT NULL
);

CREATE TABLE IF NOT EXISTS musicbrainz_tracks (
    impresario_id UUID PRIMARY KEY
        REFERENCES impresario_files (id)
        ON DELETE CASCADE,
    track_id UUID NOT NULL
);
