-- Federation: add retry backoff columns for sync error resilience

ALTER TABLE federation_peers ADD COLUMN consecutive_failures INTEGER NOT NULL DEFAULT 0;
ALTER TABLE federation_peers ADD COLUMN next_retry_at TEXT;
