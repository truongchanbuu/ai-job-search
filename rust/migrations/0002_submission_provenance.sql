-- Version 1 used the report time as submitted_at. It is not verified platform time.
ALTER TABLE applications ADD COLUMN submission_reported_at TIMESTAMPTZ;
ALTER TABLE applications DROP CONSTRAINT applications_check;
UPDATE applications SET submission_reported_at = submitted_at, submitted_at = NULL
WHERE submitted_at IS NOT NULL;
