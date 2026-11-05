CREATE TABLE worker_heartbeats (
    id UUID PRIMARY KEY,
    last_seen TIMESTAMPTZ NOT NULL
);
