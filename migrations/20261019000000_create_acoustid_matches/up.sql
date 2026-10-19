CREATE TABLE acoustid_matches (
    impresario_id UUID NOT NULL
        REFERENCES impresario_files (id)
        ON DELETE CASCADE,
    acoustid_id UUID NOT NULL,
    score REAL NOT NULL,
    PRIMARY KEY (impresario_id, acoustid_id)
);