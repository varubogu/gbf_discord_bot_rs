# CLAUDE.md

This file provides guidance to Claude Code when working with this repository.

## Project Overview

GBF Discord Bot - A Discord bot supporting Granblue Fantasy game activities.
Built with Rust + poise + PostgreSQL + SeaORM, following clean architecture.

## Common Commands

### Development
```bash
cargo build                    # Build project
cargo run                      # Run bot (requires .env)
cargo test                     # Run tests
cargo test test_name           # Run specific test
cargo test -- --nocapture      # Run with logging
```

**IMPORTANT: DO NOT run `cargo build --release` or any release builds. Development builds only.**

### Linting and Formatting
```bash
cargo clippy                   # Check code
cargo fmt                      # Format code
cargo fmt -- --check           # Check formatting
cargo run --bin schema_lint    # Verify schema consistency
```

### Database Migrations
```bash
cargo run -- migrate           # Run migrations
cd migration && sea-orm-cli migrate generate migration_name
```

## Key Technologies

- **Discord Bot**: poise 0.6.1
- **Async Runtime**: tokio 1.47
- **ORM**: SeaORM 1.1 (PostgreSQL)
- **Error Handling**: thiserror 1.0
- **Logging**: tracing 0.1 + tracing-subscriber 0.3
- **Testing**: mockall

## Architecture & Standards

This project follows clean architecture with strict layer separation.
See `.claude/skills/` for detailed guidelines:

- `clean-architecture` - Layer responsibilities and transaction management
- `coding-standards` - Error handling, logging, naming conventions
- `database` - SeaORM usage and migration guidelines
- `testing` - Test structure and mocking patterns
- `documentation` - Documentation principles for `docs/en/developer/` (fallback: `docs/ja/開発者向け/`)
- `architecture-lint` - Detect architecture violations

## Schema Management

Schema name mappings are automatically generated from entity definitions.
See `docs/en/developer/05_database/schema_management.md` for details.

- Entity schema info → auto-generated at build time
- Use `get_schema_name(table_name)` and `get_entity_table_ref(table_name)` from `schema_utils`
- Run `cargo run --bin schema_lint` to verify consistency

## Important Rules

- **All comments, docs, and error messages in code must be in Japanese**
- **All chat responses to users must be in Japanese**
- **Do not run multiple cargo commands concurrently**: Always run `cargo` commands sequentially. Start the next `cargo` command only after the previous `cargo` process has finished (no background/parallel runs).
- **All user-facing strings (message content, embeds, and component text such as labels/placeholders) must be defined in `locales/messages.yml` and retrieved via the message abstraction (`MessageTextId` / `MessageService`)**
- **Do not hardcode user-facing text in Rust source files**
- Detailed design documents in `docs/en/developer/` (fallback: `docs/ja/開発者向け/`; see documentation skill)
- If a referenced doc path does not exist (doc reorg), search under `docs/en/` first, then `docs/ja/`.
- Skills auto-trigger based on context - no need to explicitly call them

## Quality Assurance

When code modifications are required, follow this workflow in order:

1. **Documentation** - Update relevant documentation first
2. **Code** - Implement the code changes
3. **Lint** - Run `cargo clippy` to check for issues
4. **Format** - Run `cargo fmt` to format code
5. **Test** - Run targeted tests for modified code with `cargo test <test_name>`, then run full test suite with `cargo test`

All steps must be completed before considering the changes complete.
