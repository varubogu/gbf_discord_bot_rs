# Project Structure (Developer)

## Purpose

Understand “what is located where” so you don’t get lost when making changes.

## Key directories (overview)

```
(root)
├── .tmp      # Temporary workspace (not committed). Use freely for experiments.
├── db/       # DB configuration
│   ├── sh/   # Shell scripts for DB setup
│   ├── sql/  # SQL for initial seed data
├── docs/     # Documentation
├── locales/  # i18n message files
│   └── messages.yml  # Japanese messages (for rust-i18n)
├── migration/        # DB migration files
├── src            # Source code
│   ├── bin/       # Maintenance binaries (e.g., cleanup)
│   │   └── cleanup.rs   # Entry point for data cleanup batch
│   │   ├── di/        # Dependency injection (DI) container module
│   │   └── errors/    # Error definitions
│   ├── events/            # Discord events/interactions
│   │   ├── handlers/      # Event handlers (on_message, on_reaction_add, etc.)
│   │   ├── interactions/  # Interactions
│   │   │   ├── command_interactions/   # Command-based interactions
│   │   │   │   └── slash/              # Slash command definitions
│   │   │   ├── components/             # Component interactions
│   │   │   └── modal/                  # Modal interactions
│   │   └── handler.rs     # Events/interactions entry
│   ├── facades/           # Facade layer (called from events/interactions/schedulers)
│   ├── gateway/           # Interfaces between poise/serenity and business logic
│   ├── infrastructure/    # Infrastructure layer (external APIs, DB, etc.)
│   │   └── database/      # DB CRUD implementation (sea-orm based)
│   ├── models/            # Structs used by facades/services
│   │   └── entities/      # Table mapping definitions (convertible to/from models)
│   ├── presenter/         # Presenter layer (build UI, convert to poise/serenity types)

│   ├── repository/     # Persistence (save/load)
│   │   └── database/   # DB CRUD (sea-orm based)
│   ├── services/                   # Service layer (single-responsibility services)
│   │   ├── battle_recruitment/     # Co-op recruitment and notifications
│   │   ├── environment/            # Environment variable loading
│   │   ├── message/                # Message abstraction (rust-i18n + per-server overrides)
│   │   └── permission/             # Discord permission-related services
│   ├── types/        # type/enum definitions (domain objects live under src/models)
│   ├── utils/        # Shared utilities
│   ├── lib.rs        # Library crate root (mainly for test module declarations)
│   └── main.rs       # Entry point
├── target            # Cargo build outputs
├── tests             # Tests
│   ├── integration/  # Integration tests
│   ├── system/       # System tests
│   └── mod.rs        # Test module declarations
├── tools/            # Tools
│   └── schema_lint/  # Schema consistency checker
├── .env.app              # App env vars for development (contains secrets; never commit)
├── .env.app.example      # Example for .env.app
├── .env.db               # DB container env vars
├── .env.db.example       # Example for .env.db
├── .env.maintenance      # Maintenance env vars
├── .env.maintenance.example  # Example for .env.maintenance
├── Cargo.lock            # Locked dependencies
└── Cargo.toml            # Cargo config
```
