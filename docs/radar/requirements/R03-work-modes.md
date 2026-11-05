# R03 — Work modes, scheduling and quotas

**Owner:** Scheduler · **Priority:** Must · **Related:** config/radar.example.yaml

## Requirements

- **FR-020** Support `employed`, `available`, `busy`, `paused` modes with overridable quotas/schedules.
- **FR-021** Configure timezone, weekdays, scan times, max source requests, max jobs ingested, max deep evaluations, quiet hours and daily notification caps.
- **FR-022** Support `scan now`, `pause`, `resume`, and a visible last/next run. Distinguish requested run from actually executed run.
- **FR-023** Scheduler runs on cloud when owner PC is off; persist job run state and protect against overlap across process restarts.
- **FR-024** Bound retries and report provider/auth failures; skipped scans should be recorded with reasons.
- **FR-025** `paused` disables new discovery and ordinary job alerts while retaining security/critical system notifications if enabled.

## Business rules

- **BR-020** Schedules do not override upstream platform terms or permission gates.
- **BR-021** Live TUI settings create audited DB overrides; base YAML remains deployment defaults. Effective precedence: approved DB override > loaded YAML > code default.
- **BR-022** `max_jobs_per_day` means jobs processed, **not** guaranteed number of jobs returned.
- **BR-023** Quiet hours defer ordinary notifications; do not lose persisted events.

## Acceptance

- **AC-020** `paused` mode prevents the next mock scan; `resume` restores normal operation.
- **AC-021** Two scheduler workers cannot process the same scheduled scan concurrently.
- **AC-022** Updating a quota in TUI affects the next eligible scheduled run and appears in API effective config.
