-- Preserve the exact card and job revision used by interactive buttons, even after later rescoring.
ALTER TABLE notification_deliveries ADD COLUMN message JSONB;
