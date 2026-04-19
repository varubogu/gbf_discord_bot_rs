# Maintenance (Migrations and Cleanup)

This page describes safe procedures for operators to perform “DB schema updates” and “deleting old data”.

## Migrations (DB updates)

The DB schema may change when new bot features are added. Usually you run migrations when updating the bot.

### When to run

- When you are told a DB update is required right after (or before) updating the bot
- When new features require the DB but don’t work without an update

### Command

- `cargo run -- migrate`

## Data cleanup (delete old data)

This operation deletes old recruitment posts, notifications, and tasks to reduce DB growth and performance degradation.
It runs via a maintenance path separated from the main bot.

### Overview

- By default, deletes target data older than 30 days
- Runs in dedicated maintenance containers (separate from the bot)
- Uses a dedicated DB role (`gbf_bot_cleanup`) for least-privilege operation
- `maintenance_scheduler` runs continuously and executes cleanup daily in UTC time

### When to run

- During low-usage hours such as late night (recommended)
- When DB storage keeps growing

## Prerequisites

- `.env.maintenance` is configured
- PostgreSQL is running
- The cleanup execution user (`gbf_bot_cleanup`) is created

## How to run

### Development environment

- `cargo run --bin cleanup`

### Run by setting env vars directly

- Set `DB_HOST`, `DB_PORT`, `DB_USER`, `DB_PASSWORD`, `DB_NAME`
- Override `CLEANUP_RETENTION_DAYS` if needed

### Production (Docker Compose)

`./mng.sh prod up` starts `maintenance_scheduler` automatically.

### Manual one-shot execution

- `docker compose run --rm maintenance`
- `docker compose run --rm -e CLEANUP_EXECUTION_MODE=once maintenance`

### Temporarily change retention days

- `docker compose run --rm -e CLEANUP_RETENTION_DAYS=60 maintenance`

## Scheduler settings (container built-in)

Configure `.env.maintenance` to tune daily execution timing:

- `CLEANUP_SCHEDULE_HOUR_UTC` (default `3`)
- `CLEANUP_SCHEDULE_MINUTE_UTC` (default `0`)
- `CLEANUP_RUN_ON_STARTUP` (default `false`)

Notes:

- Running at night (e.g., 03:00 UTC) is recommended
- Keep log rotation enabled to avoid logs growing indefinitely

## Data targets

### 1. `worker.battle_recruitments`

- Delete condition: quest start time is older than the retention period **and** the recruitment is finished
- Related data (participants, dismissal settings, notification relations) is also deleted via FK cascades

### 2. `worker.notifications`

- Delete condition: scheduled notification time is older than the retention period **and** it has been sent
- Related notification tables are also deleted via FK cascades

### 3. `worker.scheduled_tasks`

- Delete condition: scheduled execution time is older than the retention period **and** the task is done **and** it is not `DataCleanup`
- Related task tables are also deleted via FK cascades

## Retention period

- Default: `CLEANUP_RETENTION_DAYS=30`
- Permanent change: update `.env.maintenance`

## Failure behavior

- If cleanup fails, the transaction is rolled back
- It does not leave partially-deleted state behind

## Monitoring and logs

- Check logs for start time, reference time, number of deletions, and success/failure
- Example commands:
  - `tail -f /var/log/gbf-cleanup.log`
  - `docker compose logs -f maintenance_scheduler`
  - `docker compose logs maintenance`

### Debug run

- `docker compose run --rm -e RUST_LOG=debug maintenance`

## Notes

- Avoid concurrent runs (even if idempotent, it increases operational load)
- Deleted data cannot be restored; make regular backups mandatory
- If changing retention in production, get agreement first before applying
- Before manual one-shot runs, stop `maintenance_scheduler` or ensure no overlap

## Related documents

- [DB role design](../developer/05_database/db_role_design.md)
- [Monitoring and logs](07_monitoring_and_logs.md)
- [Troubleshooting](08_troubleshooting.md)
