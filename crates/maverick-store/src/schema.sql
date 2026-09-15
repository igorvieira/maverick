CREATE TABLE IF NOT EXISTS runs (
    id TEXT PRIMARY KEY,
    task TEXT NOT NULL,
    state TEXT NOT NULL,
    revision INTEGER NOT NULL CHECK (revision >= 0),
    created_at INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS transitions (
    run_id TEXT NOT NULL REFERENCES runs(id),
    revision INTEGER NOT NULL,
    from_state TEXT NOT NULL,
    to_state TEXT NOT NULL,
    occurred_at INTEGER NOT NULL,
    PRIMARY KEY (run_id, revision)
);
CREATE TRIGGER IF NOT EXISTS transitions_no_update
BEFORE UPDATE ON transitions BEGIN
    SELECT RAISE(ABORT, 'transition history is append-only');
END;
CREATE TRIGGER IF NOT EXISTS transitions_no_delete
BEFORE DELETE ON transitions BEGIN
    SELECT RAISE(ABORT, 'transition history is append-only');
END;
CREATE TABLE IF NOT EXISTS artifacts (
    run_id TEXT NOT NULL REFERENCES runs(id),
    kind TEXT NOT NULL,
    name TEXT NOT NULL,
    revision INTEGER NOT NULL CHECK (revision > 0),
    PRIMARY KEY (run_id, kind, name, revision)
);
PRAGMA user_version = 1;
