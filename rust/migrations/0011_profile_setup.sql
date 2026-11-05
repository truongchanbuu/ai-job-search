-- Reuse immutable profile_revisions already referenced by imports/evaluations.
CREATE TABLE owner_profile (
    singleton BOOLEAN PRIMARY KEY DEFAULT true CHECK(singleton),
    revision BIGINT NOT NULL DEFAULT 0,
    active_revision TEXT REFERENCES profile_revisions(revision)
);
INSERT INTO owner_profile(singleton) VALUES(true);
