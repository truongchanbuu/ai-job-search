CREATE TABLE discord_interactions (
    provider TEXT NOT NULL,
    interaction_id TEXT NOT NULL,
    body_hash TEXT NOT NULL,
    application_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    guild_id TEXT NOT NULL,
    channel_id TEXT NOT NULL,
    message_id TEXT,
    command JSONB NOT NULL,
    token_encrypted BYTEA,
    acknowledgment JSONB NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('pending','running','completed','failed')),
    owner UUID,
    lease_until TIMESTAMPTZ,
    result JSONB,
    reply_status TEXT NOT NULL DEFAULT 'pending' CHECK(reply_status IN ('pending','sending','sent','failed','none')),
    reply_attempts INTEGER NOT NULL DEFAULT 0,
    reply_next_at TIMESTAMPTZ NOT NULL,
    reply_owner UUID,
    reply_lease_until TIMESTAMPTZ,
    error TEXT,
    received_at TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    completed_at TIMESTAMPTZ,
    PRIMARY KEY(provider,interaction_id)
);
CREATE INDEX interactions_pending ON discord_interactions(status,received_at);
CREATE INDEX interactions_replies ON discord_interactions(reply_status,reply_next_at);
