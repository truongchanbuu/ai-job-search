-- Scheduled Upwork scans are allowed only when runtime configuration opts in.
-- The application remains fail-closed when execution_mode is manual (default).
ALTER TABLE scan_runs DROP CONSTRAINT upwork_manual_only;
ALTER TABLE scan_runs ADD CONSTRAINT upwork_origin_allowed
    CHECK (source = 'mock' OR origin IN ('scheduled', 'manual', 'discord'));
