# Project Structure (Developer)

## Purpose

Understand “what is located where” so you don’t get lost when making changes.

## Key directories (overview)

```text
(root)
├── .tmp/                  # Temporary workspace (not committed)
├── db/                    # DB setup scripts / seed SQL
├── docs/                  # Documentation
├── locales/               # i18n message files
├── migration/             # DB migrations
├── src/
│   ├── bin/               # Maintenance binaries (e.g. cleanup)
│   ├── di/                # Dependency injection / composition root
│   ├── events/            # Presentation layer (Discord input)
│   ├── facades/           # Application layer (use-case + transaction boundary)
│   ├── services/          # Domain/business logic layer
│   ├── repository/        # Persistence ports (trait + DTOs only)
│   │   ├── schedule/
│   │   ├── auto_recruitment/
│   │   └── ...
│   ├── infrastructure/    # External systems / concrete adapters
│   │   └── database/
│   │       ├── connection/      # DB connection management
│   │       ├── session/         # DB session context (e.g. RLS context variables)
│   │       ├── repositories/    # SeaORM repository adapters (concrete impl)
│   │       │   ├── schedule/
│   │       │   ├── auto_recruitment/
│   │       │   └── ...
│   │       └── mod.rs
│   ├── gateway/           # Discord gateway abstraction
│   ├── models/            # Domain / entity models
│   ├── presenter/         # View model and Discord response builders
│   ├── types/             # Shared type definitions
│   ├── utils/             # Shared utilities
│   ├── lib.rs
│   └── main.rs
├── tests/
├── tools/
└── Cargo.toml
```

## Repository and Infrastructure responsibilities

- `src/repository/**` is the **port layer**.
  - Defines traits and request/response structs used by services.
  - Must not contain ORM-specific details.
- `src/infrastructure/database/repositories/**` is the **adapter layer**.
  - Implements repository traits with SeaORM.
  - Owns SQL/ORM behavior and DB-specific optimization.

## Temporary compatibility policy (during migration)

During the staged repository refactor:

- Old paths under `src/repository/database/**` may remain temporarily as compatibility re-exports.
- New development should reference:
  - Port: `src/repository/**`
  - Adapter: `src/infrastructure/database/repositories/**`
- Compatibility re-exports are transitional and will be removed in the final cleanup phase.
