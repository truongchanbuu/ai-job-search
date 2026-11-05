ALTER TABLE feedback DROP CONSTRAINT feedback_action_check;
ALTER TABLE feedback ADD CONSTRAINT feedback_action_check CHECK(action IN ('interested','save','skip','not_available','applied','interview','hired','rejected','completed','withdrawn','other'));
ALTER TABLE applications DROP CONSTRAINT applications_status_check;
ALTER TABLE applications ADD CONSTRAINT applications_status_check CHECK(status IN ('none','drafted','submitted','interviewing','hired','rejected','completed','withdrawn'));
ALTER TABLE feedback ADD COLUMN reason_code TEXT;
ALTER TABLE feedback ADD COLUMN learning_excluded BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE feedback ADD COLUMN learning_version BIGINT NOT NULL DEFAULT 1;
ALTER TABLE feedback ADD COLUMN actual_effort_hours DOUBLE PRECISION CHECK(actual_effort_hours>0 AND actual_effort_hours<=10000);
ALTER TABLE feedback ADD COLUMN learning_snapshot JSONB;
CREATE TABLE feedback_learning_revisions (
    feedback_id UUID NOT NULL REFERENCES feedback(id) ON DELETE CASCADE,
    version BIGINT NOT NULL,
    reason_code TEXT,
    excluded BOOLEAN NOT NULL,
    actual_effort_hours DOUBLE PRECISION CHECK(actual_effort_hours>0 AND actual_effort_hours<=10000),
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY(feedback_id,version)
);
INSERT INTO feedback_learning_revisions(feedback_id,version,excluded,created_at)
SELECT id,1,false,created_at FROM feedback;
CREATE TABLE preference_models (
    id UUID PRIMARY KEY,
    request_hash TEXT NOT NULL,
    snapshot JSONB NOT NULL,
    rejected BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp()
);
CREATE TABLE preference_state (
    singleton BOOLEAN PRIMARY KEY DEFAULT true CHECK(singleton),
    revision BIGINT NOT NULL DEFAULT 0,
    feedback_revision BIGINT NOT NULL DEFAULT 0,
    active_model UUID REFERENCES preference_models(id),
    suppressed JSONB NOT NULL DEFAULT '[]'
);
INSERT INTO preference_state(singleton) VALUES(true);
