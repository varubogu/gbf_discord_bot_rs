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

## Production Deployment

This project uses GitHub Actions to build Docker images and push them to GitHub Container Registry (GHCR). The production server only pulls pre-built images, avoiding resource-intensive builds.

### Setup Steps

1. **Enable GitHub Container Registry**
   - Go to your repository Settings > Actions > General
   - Enable "Read and write permissions" for GITHUB_TOKEN

2. **Configure Production Server**
   ```bash
   # Clone the repository
   git clone https://github.com/your-username/gbf_discord_bot_rs.git
   cd gbf_discord_bot_rs

   # Set GITHUB_REPOSITORY environment variable
   export GITHUB_REPOSITORY=your-username/gbf_discord_bot_rs

   # Login to GitHub Container Registry
   echo $GITHUB_TOKEN | docker login ghcr.io -u your-username --password-stdin

   # Create environment files
   cp .env.app.example .env.app
   cp .env.db.example .env.db
   # Edit .env.app and .env.db with your configuration

   # Create .local directory for Google Service Account Key
   mkdir -p .local
   # Place your service account key file in .local/

   # Pull and start services
   docker-compose pull
   docker-compose up -d
   # or use the management script
   ./mng.sh prod up
   ```

3. **Automatic Deployment**
   - Push to `main` branch triggers automatic build and push to GHCR
   - On production server, pull and restart:
   ```bash
   docker-compose pull
   docker-compose up -d
   # or use the management script
   ./mng.sh prod up
   ```

### Development vs Production

- **Development**: Use `./mng.sh dev up` to run database locally with Docker
- **Production** (`docker-compose.yml`): Pulls pre-built images from GHCR (no local build)

## License

Please contact the project maintainers for license information.