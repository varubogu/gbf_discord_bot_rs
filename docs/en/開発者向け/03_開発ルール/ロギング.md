# Logging

## Principles

- Use `tracing` and emit structured logs
- Do not log secrets
- Use `warn` for expected business exceptions, and `error` for system failures
## Useful fields to log

- `guild_id` / `channel_id` / `user_id` (where possible)
- The use case (e.g., create recruitment, load spreadsheet)
- Where external failures occurred (Discord/DB/Spreadsheet)
