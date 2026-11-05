ALTER TABLE scan_runs ADD COLUMN source TEXT NOT NULL DEFAULT 'mock' CHECK(source IN ('mock','upwork'));
ALTER TABLE scan_runs ADD COLUMN search_profile TEXT;
ALTER TABLE scan_runs ADD CONSTRAINT upwork_manual_only CHECK(source='mock' OR origin IN ('manual','discord'));
CREATE TABLE upwork_source_state (
    singleton BOOLEAN PRIMARY KEY DEFAULT true CHECK(singleton),
    snapshot JSONB NOT NULL DEFAULT '{}'
);
INSERT INTO upwork_source_state(singleton) VALUES(true);

-- Canonical provider provenance differs from the historical disabled capability label.
INSERT INTO job_sources(name,enabled,diagnostic) VALUES ('upwork',false,'manual mode only; configuration and Radar OAuth required') ON CONFLICT DO NOTHING;
