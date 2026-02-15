# Environment Setup (Developer)

## Prerequisites

- Rust (the toolchain specified by this project)
- PostgreSQL (you can use DevContainer or Docker)
- Discord bot token
- Google Sheets (global + per-server)
- Google service account key (JSON)

## Shortest path (DevContainer)

1. Copy `.env` templates and fill in values
   - `.env.app.example` → `.env.app`
   - `.env.db.example` → `.env.db`
2. Place the service account key JSON under `.local/`, then update `GOOGLE_SERVICE_ACCOUNT_KEY_FILE` in `.env.app`
3. Start DevContainer (PostgreSQL is expected to start together)
4. Start the bot with `cargo run`

## Notes for local runs

- `.env` files contain secrets. Never commit them (they are already in `.gitignore`).
- If you see errors, first check logs and DB connectivity (`DATABASE_URL`).
