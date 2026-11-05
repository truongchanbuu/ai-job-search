CREATE TABLE outbox_events (
    id UUID PRIMARY KEY,
    event_key TEXT NOT NULL UNIQUE,
    kind TEXT NOT NULL CHECK(kind IN ('job_matched','daily_digest','scan_failed')),
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    processed_at TIMESTAMPTZ
);
CREATE TABLE notification_deliveries (
    id UUID PRIMARY KEY,
    event_id UUID NOT NULL REFERENCES outbox_events(id),
    destination TEXT NOT NULL,
    provider TEXT NOT NULL,
    channel_id TEXT,
    status TEXT NOT NULL DEFAULT 'pending' CHECK(status IN ('pending','sending','retry','sent','failed','uncertain','cancelled')),
    attempts INTEGER NOT NULL DEFAULT 0,
    next_attempt_at TIMESTAMPTZ NOT NULL,
    lease_until TIMESTAMPTZ,
    owner UUID,
    budget_date DATE,
    remote_id TEXT,
    error TEXT,
    delivered_at TIMESTAMPTZ,
    UNIQUE(event_id,destination)
);
CREATE INDEX deliveries_due ON notification_deliveries(status,next_attempt_at);
CREATE TABLE notification_budget (
    local_date DATE PRIMARY KEY,
    reserved INTEGER NOT NULL DEFAULT 0 CHECK(reserved>=0)
);
CREATE TABLE provider_cooldowns (
    provider TEXT NOT NULL,
    scope TEXT NOT NULL,
    until_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY(provider,scope)
);
CREATE TABLE digest_state (
    singleton BOOLEAN PRIMARY KEY DEFAULT true CHECK(singleton),
    planned_through TIMESTAMPTZ
);
INSERT INTO digest_state(singleton) VALUES(true);
