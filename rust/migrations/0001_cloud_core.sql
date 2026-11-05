CREATE TABLE config_revisions (
    revision BIGINT PRIMARY KEY CHECK (revision > 0),
    snapshot JSONB NOT NULL,
    actor TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE runtime_config (
    singleton BOOLEAN PRIMARY KEY DEFAULT true CHECK (singleton),
    revision BIGINT NOT NULL REFERENCES config_revisions(revision),
    effective JSONB NOT NULL,
    overrides JSONB NOT NULL DEFAULT '{}',
    baseline_hash TEXT NOT NULL,
    actor TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE profile_revisions (
    revision TEXT PRIMARY KEY,
    snapshot JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE job_sources (
    name TEXT PRIMARY KEY,
    enabled BOOLEAN NOT NULL,
    diagnostic TEXT NOT NULL
);
CREATE TABLE imports (
    id UUID PRIMARY KEY,
    request_hash TEXT NOT NULL,
    config_revision BIGINT NOT NULL REFERENCES config_revisions(revision),
    profile_revision TEXT NOT NULL REFERENCES profile_revisions(revision),
    response JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE jobs (
    id UUID PRIMARY KEY,
    canonical_identity TEXT NOT NULL UNIQUE,
    provider TEXT NOT NULL REFERENCES job_sources(name),
    source_id TEXT,
    title TEXT NOT NULL,
    payload JSONB NOT NULL,
    fetched_at TIMESTAMPTZ NOT NULL,
    decision TEXT CHECK (decision IN ('interested','save','skip','not_available')),
    revision BIGINT NOT NULL DEFAULT 0 CHECK (revision >= 0)
);
CREATE UNIQUE INDEX jobs_source_identity ON jobs(provider, source_id) WHERE source_id IS NOT NULL;
CREATE TABLE job_ingestions (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    job_id UUID NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    import_id UUID NOT NULL REFERENCES imports(id),
    payload_hash TEXT NOT NULL,
    input JSONB NOT NULL,
    fetched_at TIMESTAMPTZ NOT NULL
);
CREATE INDEX job_ingestions_job ON job_ingestions(job_id);
CREATE TABLE evaluations (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    job_id UUID NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    import_id UUID NOT NULL REFERENCES imports(id),
    config_revision BIGINT NOT NULL REFERENCES config_revisions(revision),
    profile_revision TEXT NOT NULL REFERENCES profile_revisions(revision),
    input JSONB NOT NULL,
    snapshot JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    UNIQUE (job_id, import_id)
);
CREATE INDEX evaluations_latest ON evaluations(job_id, id DESC);
CREATE TABLE applications (
    job_id UUID PRIMARY KEY REFERENCES jobs(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'none' CHECK (status IN ('none','drafted','submitted','interviewing','hired','rejected','completed')),
    submitted_at TIMESTAMPTZ,
    connects_spent INTEGER CHECK (connects_spent >= 0),
    CHECK (status IN ('none','drafted') OR submitted_at IS NOT NULL)
);
CREATE TABLE feedback (
    id UUID PRIMARY KEY,
    job_id UUID NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    request_hash TEXT NOT NULL,
    action TEXT NOT NULL CHECK (action IN ('interested','save','skip','not_available','applied','interview','hired','rejected','completed')),
    reason TEXT,
    actor TEXT NOT NULL,
    revision BIGINT NOT NULL,
    application_status TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (job_id, revision)
);
CREATE INDEX feedback_latest ON feedback(created_at DESC, id);
CREATE TABLE daily_usage (
    local_date DATE PRIMARY KEY,
    requests BIGINT NOT NULL CHECK (requests >= 0),
    jobs BIGINT NOT NULL CHECK (jobs >= 0)
);
CREATE TABLE audit_log (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    actor TEXT NOT NULL,
    action TEXT NOT NULL,
    entity TEXT NOT NULL,
    details JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
