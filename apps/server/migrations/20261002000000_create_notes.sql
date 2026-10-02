CREATE TABLE notes (
    id         INTEGER PRIMARY KEY,
    body       TEXT NOT NULL CHECK (length(body) BETWEEN 1 AND 1000),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
) STRICT;
