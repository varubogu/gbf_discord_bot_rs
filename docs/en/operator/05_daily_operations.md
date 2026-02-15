# Day-to-day Operations (Operator)

## Daily/weekly checklist (example)

- Confirm the bot is running (commands respond in Discord, or check logs).
- Confirm errors are not increasing.

## Updates

Steps vary by your operational approach.

- If using containers: update the image and restart.
- Otherwise: follow your procedure (containerization is recommended if possible).

## Restart (the first thing to try)

- A restart often resolves issues.
- If not, check logs and DB connectivity.

## Backups (recommended)

Take regular backups in case the DB is corrupted or lost.
Because methods vary by environment, decide “where / how often / who” among operators first.

## Read next

- [Monitoring and logs](07_monitoring_and_logs.md)
- [Maintenance](06_maintenance_migrations_and_cleanup.md)
