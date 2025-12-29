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
- `documentation` - Documentation principles for `docs/develop/`
- `architecture-lint` - Detect architecture violations

## Schema Management

Schema name mappings are automatically generated from entity definitions.
See `docs/schema_management.md` for details.

- Entity schema info → auto-generated at build time
- Use `get_schema_name(table_name)` and `get_entity_table_ref(table_name)` from `schema_utils`
- Run `cargo run --bin schema_lint` to verify consistency

## Important Rules

- **All comments, docs, and error messages in code must be in Japanese**
- **All chat responses to users must be in Japanese**
- Detailed design documents in `docs/develop/` (see documentation skill)
- Skills auto-trigger based on context - no need to explicitly call them
