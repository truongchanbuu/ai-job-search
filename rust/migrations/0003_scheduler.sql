CREATE TABLE scheduler_state (
    singleton BOOLEAN PRIMARY KEY DEFAULT true CHECK(singleton),
    planned_through TIMESTAMPTZ,
    config_revision BIGINT,
    previous_mode TEXT
);
INSERT INTO scheduler_state(singleton) VALUES(true);
CREATE TABLE scan_runs (
    id UUID PRIMARY KEY,
    run_key TEXT NOT NULL UNIQUE,
    origin TEXT NOT NULL CHECK(origin IN ('scheduled','manual','discord')),
    scheduled_at TIMESTAMPTZ NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending' CHECK(status IN ('pending','running','completed','failed')),
    owner UUID,
    lease_until TIMESTAMPTZ,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    attempts INTEGER NOT NULL DEFAULT 0,
    result JSONB,
    error TEXT
);
CREATE UNIQUE INDEX scan_one_running ON scan_runs((true)) WHERE status='running';
CREATE INDEX scan_pending ON scan_runs(status,scheduled_at);
CREATE TABLE mode_requests (
    id UUID PRIMARY KEY,
    paused BOOLEAN NOT NULL,
    response JSONB NOT NULL
);
