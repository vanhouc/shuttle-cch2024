-- Add migration script here
CREATE TABLE IF NOT EXISTS cursors 
(
    id BIGSERIAL PRIMARY KEY,
    token TEXT NOT NULL,
    page INT NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL
);