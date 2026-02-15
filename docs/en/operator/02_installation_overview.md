# Installation Steps (Bot Operator)

This page summarizes the shortest path to “run the bot for the first time”.

## What you need

- Discord bot token
- PostgreSQL (DB)
- Google Sheets (global + per-server)
- Google service account key (JSON)

## Terms

- **Discord bot token**: The “key” to run the bot. Treat it as secret; if leaked, your bot can be hijacked.
- **DB (database)**: Stores recruitment data and settings.
- **Service account key (JSON)**: The key for accessing spreadsheets. Create and use a Google Cloud service account key.

## Recommended approach

- Avoid heavy builds directly on production servers.
- If possible, use **containers (Docker Compose)** to simplify start/update.

## Steps (overview)

1. Prepare config files (`.env.app`, `.env.db`, `.env.maintenance`).
2. Put the service account key JSON under `.local/`, and set its path in `.env.app`.
3. Start the DB (Docker Compose recommended).
4. Start the bot (Docker Compose recommended).
5. Do initial configuration on Discord (role/spreadsheet/channel registration).

Note: Step 5 is the server administrator’s task. See the separate guide.

- [Server administrator documentation](../server_administrator_guild_master/README.md)

## One-time tasks (easy to forget)

- DB migration (initialize/update schema)
- Set the admin server ID (if using admin commands)

## Read next

- [Configuration (environment variables)](03_installation_configuration_details.md)
- [Start and update (Docker Compose)](10_start_and_update_docker_compose.md)
