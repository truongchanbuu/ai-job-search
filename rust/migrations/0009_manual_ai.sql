CREATE TABLE ai_operations (
    id UUID PRIMARY KEY,
    job_id UUID NOT NULL REFERENCES jobs(id),
    kind TEXT NOT NULL CHECK (kind IN ('evaluate','proposal')),
    request_hash TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('running','completed','failed','cancelled')),
    snapshot JSONB NOT NULL,
    output JSONB,
    usage JSONB NOT NULL DEFAULT '{}',
    approved BOOLEAN NOT NULL DEFAULT false,
    error TEXT,
    reserved_cost DOUBLE PRECISION NOT NULL CHECK (reserved_cost >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    lease_until TIMESTAMPTZ NOT NULL,
    completed_at TIMESTAMPTZ
);
CREATE UNIQUE INDEX ai_one_active_kind_per_job ON ai_operations(job_id,kind) WHERE status='running';
CREATE INDEX ai_operations_daily ON ai_operations(created_at);
CREATE TABLE proposal_revisions (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    job_id UUID NOT NULL REFERENCES jobs(id),
    operation_id UUID NOT NULL REFERENCES ai_operations(id),
    revision BIGINT NOT NULL,
    ai_text TEXT NOT NULL,
    user_text TEXT,
    approved_text TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(job_id,revision)
);
