# AGENTS.md

## Purpose
Keep this file as the always-on, low-token contract for the repository.
Put task-specific detail in repo-local Skills under `.codex/skills/`.

## Always-On Rules
- Stack: Rust + poise + SeaORM + PostgreSQL
- Architecture: strict one-way dependencies `events → facades → services → repository`
- **Only Facades** start/commit/rollback transactions
- Services receive transactions and pass them to Repositories
- DB connections come from `AppState`; do not create ad-hoc connections per layer
- Comments, design docs, and error messages in code must be Japanese; identifiers stay English
- User-facing strings must live in `locales/messages.yml` and be accessed via `MessageTextId` / `MessageService`
- Every change should update tests and the related docs / locales / migrations when applicable
- If rules conflict, follow the most specific design doc

## Standard Flow
1. Read the relevant docs/specs first.
2. Update design or feature docs before code when behavior/design changes.
3. Implement code.
4. Run `cargo clippy -j 1`.
5. Run `cargo fmt`.
6. Run focused tests, then `cargo test -j 1`.

## Skills To Use
- `clean-architecture`: Layer responsibilities, dependency direction, transaction boundaries, `AppState` usage
- `coding-standards`: Japanese comments, naming, `thiserror`, `tracing`, localization rules, security, `unwrap()` avoidance
- `database`: SeaORM, migrations, schema changes, repository-layer DB patterns
- `testing`: AAA, mockall, unit/integration tests, test commands
- `documentation`: docs structure, abstraction level, permanence, docs-first updates, impact review
- `architecture-lint`: Review dependency shortcuts and transaction-boundary violations

## Key References
- `CLAUDE.md`
- `.cursor/rules/rules.mdc`
- `docs/en/developer/02_architecture/`
- `docs/en/developer/03_development_rules/`
- `docs/en/developer/05_database/`
- `docs/en/developer/06_feature_specifications/`
- If a referenced path moved, search `docs/en/` first and `docs/ja/` second

## Commands
```bash
cargo build -j 1
cargo run -j 1
cargo test -j 1
cargo test -j 1 test_name
cargo test -j 1 -- --nocapture
cargo clippy -j 1
cargo fmt
cargo fmt -- --check
cargo run -j 1 --bin schema_lint
cargo run -j 1 -- migrate
cd migration && sea-orm-cli migrate generate migration_name
```

- Never run `cargo build --release` or other release builds in this repo
- Do not run multiple `cargo` commands concurrently; always run them sequentially
