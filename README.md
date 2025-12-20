# GBF Discord Bot (Rust)

English | [日本語](README.ja.md)

A Discord bot to support Granblue Fantasy (GBF) game activities.

## Features

- Multi-battle recruitment system with element-based reactions (or buttons), including notification and scheduled recruitment features
- Event notification system for occasions like Guild War start times
- Spreadsheet integration for data injection and visualization

For more details, please refer to [docs/user](User Manual) *Japanese only*

## Requirements

- Rust 1.70+
- PostgreSQL database
- Discord bot token

## Setup

1. Clone the repository
2. Copy `.env.example` to create a `.env` file and configure environment variables
3. Run `cargo build --release`
4. Run `./target/release/gbf_discord_bot_rs`

## Commands

- `/recruit quest:<quest_name> [battle_type:<type>] [event_date:<date>]` - Create a battle recruitment

## Architecture

This project adopts Clean Architecture with clear layer-based responsibilities:

```
events (Presentation) → facades (Application) → services (Business Logic) → repository (Data Access)
```

See [CLAUDE.md](CLAUDE.md) for more details.

## Key Technologies

- **Discord Bot Framework**: poise 0.6.1
- **Async Runtime**: tokio 1.47 (multi-thread)
- **ORM**: SeaORM 1.1 (PostgreSQL)
- **Error Handling**: thiserror 1.0
- **Logging**: tracing 0.1 + tracing-subscriber 0.3
- **Testing**: tokio-test, mockall

## Development

### Build and Run

```bash
# Build the project
cargo build

# Build for release
cargo build --release

# Run the bot (requires .env configuration)
cargo run

# Run tests
cargo test

# Run a specific test
cargo test test_name
```

### Linting and Formatting

```bash
# Check code with Clippy
cargo clippy

# Format code
cargo fmt

# Check formatting without modifying files
cargo fmt -- --check
```

### Database Migrations

```bash
# Run migrations
cargo run -- migrate

# Create a new migration
cd migration
sea-orm-cli migrate generate migration_name
```

## License

Please contact the project maintainers for license information.