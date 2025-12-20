# GBF Discord Bot (Rust)

English | [日本語](README.ja.md)

A Discord bot to support Granblue Fantasy (GBF) game activities. Originally implemented in Python with discord.py, now reimplemented in Rust.

## Features

- Battle recruitment system with reactions for different elements
- Database integration for storing quest information and battle recruitments
- Slash command support
- Emoji replacement feature for multi-battle recruitment

## Requirements

- Rust 1.70+
- PostgreSQL database
- Discord bot token

## Setup

1. Clone the repository
2. Create a `.env` file in the config folder with the following variables:
   ```
   DISCORD_TOKEN=your_discord_bot_token
   GUILD_ID=your_discord_guild_id
   DB_HOST=localhost
   DB_PORT=5432
   DB_NAME=gbf_bot
   GUILD_DB_USER=guild_user
   GUILD_DB_PASSWORD=your_guild_password
   SYSTEM_DB_USER=system_user
   SYSTEM_DB_PASSWORD=your_system_password
   GLOBAL_DB_USER=global_user
   GLOBAL_DB_PASSWORD=your_global_password
   ADMIN_DB_USER=admin_user
   ADMIN_DB_PASSWORD=your_admin_password
   CONFIG_FOLDER=path_to_config_folder
   ```
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

## Migration Notes

### Key Differences Between Python and Rust Implementations

1. **Architecture**
   - Python: Uses discord.py's Cog system for organizing commands
   - Rust: Uses Clean Architecture with modular layers

2. **Database Interaction**
   - Python: Uses SQLAlchemy ORM
   - Rust: Uses SeaORM

3. **Command Handling**
   - Python: Mix of prefix commands and slash commands
   - Rust: Exclusively uses slash commands via poise framework

4. **Error Handling**
   - Python: Mix of try/except blocks and error propagation
   - Rust: Uses Rust's Result type for consistent error handling

5. **Concurrency**
   - Python: Uses asyncio for asynchronous operations
   - Rust: Uses tokio for asynchronous runtime with stronger compile-time guarantees

### Migration Challenges

1. **API Differences**: Discord.py and poise have different API designs, requiring significant adaptation
2. **Type System**: Rust's strict type system required more explicit handling of optional values and error cases
3. **Database Integration**: Moving from SQLAlchemy to SeaORM
4. **Asynchronous Programming**: Different approaches to async/await between Python and Rust

### Benefits of Rust Implementation

1. **Performance**: Rust's zero-cost abstractions provide better performance
2. **Safety**: Rust's ownership system prevents many common bugs
3. **Concurrency**: Safer concurrent code with compile-time guarantees
4. **Maintainability**: Strong type system catches many errors at compile time

## Future Improvements

1. Implement autocomplete for quest names
2. Add more commands from the original Python bot
3. Improve error handling and user feedback
4. Add tests for core functionality

## License

Please contact the project maintainers for license information.