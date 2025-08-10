# GBF Discord Bot (Rust)

This is a Rust implementation of the GBF Discord Bot, originally written in Python using discord.py. The bot is designed to help manage Granblue Fantasy game activities in Discord servers.

## Features

- Battle recruitment system with reactions for different elements
- Database integration for storing quest information and battle recruitments
- Slash command support

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
   DATABASE_URL=postgresql://user:password@localhost/gbf_bot
   CONFIG_FOLDER=path_to_config_folder
   ```
3. Run `cargo build --release`
4. Run `./target/release/gbf_discord_bot_rs`

## Commands

- `/recruit quest:<quest_name> [battle_type:<type>] [event_date:<date>]` - Create a battle recruitment

## Migration Notes

### Key Differences Between Python and Rust Implementations

1. **Architecture**
   - Python: Uses discord.py's Cog system for organizing commands
   - Rust: Uses a modular approach with separate modules for commands and database interactions

2. **Database Interaction**
   - Python: Uses SQLAlchemy ORM
   - Rust: Uses SQLx for direct SQL queries

3. **Command Handling**
   - Python: Mix of prefix commands and slash commands
   - Rust: Exclusively uses slash commands via serenity's command system

4. **Error Handling**
   - Python: Mix of try/except blocks and error propagation
   - Rust: Uses Rust's Result type for consistent error handling

5. **Concurrency**
   - Python: Uses asyncio for asynchronous operations
   - Rust: Uses tokio for asynchronous runtime with stronger compile-time guarantees

### Migration Challenges

1. **API Differences**: Discord.py and serenity have different API designs, requiring significant adaptation
2. **Type System**: Rust's strict type system required more explicit handling of optional values and error cases
3. **Database Integration**: Moving from an ORM to direct SQL queries required more manual mapping
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