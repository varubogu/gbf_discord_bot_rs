# AGENTS.md

## Purpose
This document defines how AI agents should behave when working in the GBF Discord Bot (Rust) repository.
It consolidates rules from `CLAUDE.md`, `.cursor/rules/rules.mdc`, and `docs/en/developer/**` into a compact guide.

## Key References
- `CLAUDE.md`: Core development workflow and common commands
- `.cursor/rules/rules.mdc`: Global coding and architecture rules
- `docs/en/developer/02_architecture/`: Project structure and architecture principles
- `docs/en/developer/03_development_rules/`: Coding, testing, security, performance rules
- `docs/en/developer/05_database/`: DB connection, schema, and transaction design
- `docs/en/developer/06_feature_specifications/`: Feature-level requirements and design
  - If a referenced path does not exist (doc reorg), search under `docs/en/` first, then `docs/ja/`.

## Project Summary
- Tech stack: Rust + poise + SeaORM + PostgreSQL
- Architecture: strict one-way dependencies `events → facades → services → repository`
- Dependencies are built via `AppState` and shared DB connection
- **All comments/docs/error messages in code must be Japanese; code identifiers are in English**

## Commands & Workflow

### Development Commands
```bash
cargo build                    # Build (dev only)
cargo run                      # Run bot (.env required)
cargo test                     # Run tests
cargo test test_name           # Run specific test
cargo test -- --nocapture      # Run tests with logging
```
- **Never run**: `cargo build --release` or other release builds in this repo.

### Lint / Format
```bash
cargo clippy                   # Lint
cargo fmt                      # Format
cargo fmt -- --check           # Format check only
cargo run --bin schema_lint    # Schema consistency check
```

### Database Migrations
```bash
cargo run -- migrate
cd migration && sea-orm-cli migrate generate migration_name
```

### Quality Flow (must follow in this order)
1. **Docs**: Update relevant design/feature docs first.
2. **Code**: Implement or modify code.
3. **Lint**: Run `cargo clippy`.
4. **Format**: Run `cargo fmt`.
5. **Test**: Run focused tests, then `cargo test` for full suite.

## Core Rules

### Language & Style
- Code comments, design docs, and error messages: **Japanese**
- Naming: Types `PascalCase`, functions/vars `snake_case`, consts `SCREAMING_SNAKE_CASE`
- No `unwrap()` in production; `panic!()` only for unrecoverable failures

### Architecture & Responsibilities
- No cross-layer shortcuts (e.g. Events → Services/Repository, Facade → Repository)
- **Only Facades** start/commit/rollback transactions
- Services receive a transaction and pass it to Repositories
- Repositories contain no business logic, only persistence
- DB connections are obtained via `AppState`; no ad-hoc connections per layer

### Errors & Logging
- Use `thiserror` for layer-specific error types and `#[from]` conversions
- Use `tracing` for structured logs (error / warn / info / debug)
- Do not log secrets; business exceptions are `warn`, system failures `error`

### Performance & Design
- Avoid unnecessary `clone()`, prefer borrowing
- Use `Arc<T>` only when needed
- Use concurrency (`try_join_all`) where appropriate but do not hold long transactions
- Prefer small, single-responsibility functions (≤100 lines, ≤5 nesting levels)

### Security
- Always validate inputs in the presentation layer (regex/whitelists/type checks)
- Check Discord permissions and app-level permissions
- Use SeaORM query builder; avoid raw SQL except safe prepared statements
- Sanitize Discord output where needed

### Testing
- Provide unit tests for each layer and integration tests at Facade/DB level
- Use AAA pattern (Arrange–Act–Assert)
- Regularly run `cargo test`, `cargo clippy`, `cargo fmt`

### Workflow
- Every change must include tests and documentation updates where applicable (`docs/`, `locales/`, migrations, etc.)
- Use branch names like `feature/...`, `fix/...`, `refactor/...`, `remove/...`, `docs/...`

## Agent Guidelines

### Implementation Agents (e.g. Coding, Fixer)
1. Check relevant `docs/en/developer/06_feature_specifications/` and rule docs for requirements and impact.
2. Reconfirm layer responsibilities and transaction boundaries before coding.
3. Add Japanese comments, `thiserror` errors, and `tracing` logs for new code.
4. Add/update unit and integration tests, including mocks when needed.
5. After changes, run `cargo fmt`, `cargo clippy`, and appropriate `cargo test` commands.

### Review Agents
1. Verify changes follow clean architecture and transaction rules.
2. Check error types, logging, input validation, and security measures.
3. Confirm test coverage and doc updates.
4. Watch for performance issues (unnecessary `clone`, excessive `Arc`, long transactions).
5. Provide concrete improvement suggestions with references to relevant docs.

### Documentation Agents
1. Identify affected design docs and update `docs/en/developer/02_architecture/` and/or `docs/en/developer/06_feature_specifications/`.
2. Keep abstraction high-level (responsibilities/flows/constraints, not code).
3. Ensure consistency with rule files.
4. Consider impacts on user docs (`docs/en/user/` / `docs/ja/利用者向け/`) and `locales/`.
5. Add diagrams (e.g. Mermaid) where flows benefit from visualization.

## Checklist for Any Change
- [ ] Read relevant design/rule docs
- [ ] Respect layer responsibilities and transaction rules
- [ ] Implement `thiserror` + `tracing` correctly
- [ ] Run `cargo fmt`, `cargo clippy`, `cargo test`
- [ ] Update docs / localization / migrations as needed
- [ ] Evaluate security and performance impact

## When in Doubt
- Prefer `docs/en/developer/02_architecture/` and `docs/en/developer/05_database/` for design intent
- Use `docs/en/developer/06_feature_specifications/` for feature specs and align implementation to them
- If rules conflict, follow the most specific design doc and update docs if necessary
- For undefined behavior or external dependencies, propose a design doc update or GitHub issue

## job-specific notes

When using cargo commands that trigger builds, always specify `-j 1` as a command argument to reduce machine load:

```bash
cargo check -j 1
cargo build -j 1
cargo run -j 1
cargo clippy -j 1
cargo fmt
```

This limits parallel compilation jobs to 2, preventing excessive CPU and memory usage.
