# Configuration (Environment Variables)

Environment variables decide “where the bot connects and how it behaves”.
If this is difficult, start by copying the template files (example) and filling in the blanks.

## Common terms

- **Environment variable**: A mechanism to provide app configuration values (passwords, etc.) separately from code.
- **.env file**: A way to write environment variables into a file and load them together.

## Typical settings

- `DISCORD_TOKEN`: Discord bot token
- `DATABASE_URL`: DB connection URL
- `GLOBAL_SPREADSHEET_ID`: Spreadsheet ID for global data
- `BOT_ADMIN_SERVER_ID`: Admin server ID (where admin commands run)
- `GOOGLE_SERVICE_ACCOUNT_KEY_FILE`: Path to the service account key (JSON)

## Important

These files contain secrets. If leaked, attackers may hijack the bot account, send spam messages, or tamper with spreadsheet data.
Never share or publish them.

## Common files

- `.env.app`: Main bot configuration (Discord, spreadsheets, etc.)
- `.env.db`: DB container configuration (passwords, etc.)
- `.env.maintenance`: Maintenance configuration (cleanup, etc.)
- `.local/****.json`: Google service account key (JSON)

## Quick start (shortest path)

1. Copy the example env files (`.env.xxxxx.example`) and rename them to `.env.xxxxx`.
2. Fill in the blanks (tokens/passwords/IDs).
3. Start the bot and verify it works.
