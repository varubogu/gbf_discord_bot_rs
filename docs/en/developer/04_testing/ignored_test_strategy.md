# Ignored Test Execution Strategy

Last updated: 2026-03-24

## Purpose

- Classify the large number of `#[ignore]` tests and separate them into dedicated execution lanes
- Keep important but heavy regression checks running continuously in CI

## Current classification snapshot

- Snapshot date: 2026-03-03
- Total: `145` (`rg -n "#\\[ignore\\]" tests src | wc -l`)

| Category | Count | Representative paths | Execution lane |
|---|---:|---|---|
| Real DB required | 138 | `tests/integration/facades/*`, `tests/data_cleanup_test.rs` | `ignored-db-tests` |
| DB connection required but no external dependency | 3 | Part of `tests/integration/facades/guild_settings_test.rs` | `ignored-db-tests` |
| External API or token required | 2 | `tests/system/bot_startup_test.rs` | Manual execution only |
| Other | 2 | Future classification required when added | Decide when added |

Promoted to the regular lane already (with explicit in-test skip when DB config is unavailable):

- `integration::facades::recruitment_new_test::test_update_message_id_not_found`
- `integration::facades::recruitment_new_test::test_new_recruitment_quest_not_found`
- `integration::facades::recruitment_schedule_test::test_create_schedule_basic`
- `integration::facades::recruitment_schedule_test::test_create_schedule_quest_not_found`
- `integration::facades::recruitment_schedule_test::test_create_schedule_invalid_time_format`

## Operational rules

1. If a test uses `#[ignore]`, add the reason on the same line as a comment.
2. Tests that require a real DB must run in the `ignored-db-tests` lane on a schedule.
3. Tests that require Discord tokens or other external secrets must stay out of the regular CI lane and be limited to manual execution.
4. Once an ignored test becomes stable enough, promote it to the regular lane by removing `#[ignore]`.

## CI lanes

- Regular lane: `.github/workflows/ci.yml`
  - `cargo fmt -- --check`
  - `cargo clippy -j 1`
  - `cargo test -j 1`
- Workflow: `.github/workflows/ignored-db-tests.yml`
- Preconditions:
  - A PostgreSQL service container is available
  - `cargo run -j 1 -- migrate-only` has already been executed
  - Test connections use role-specific credentials: `SYSTEM_DB_*`, `GUILD_DB_*`, `GLOBAL_DB_*`, `ADMIN_DB_*`
  - Ignored tests do not fall back to the default `DB_USER` / `DB_PASSWORD`
- Schedule:
  - Daily at 02:00 UTC
  - Manual execution via `workflow_dispatch`
- Current representative targets:
  - `integration::facades::spreadsheet_test`
  - `integration::facades::guild_settings_test`

## Local execution examples

```bash
# Equivalent to the regular lane
cargo fmt -- --check
cargo clippy -j 1
cargo test -j 1

# Ignored tests that require a DB
cargo test -j 1 --test mod integration::facades::spreadsheet_test -- --ignored
cargo test -j 1 --test mod integration::facades::guild_settings_test -- --ignored
```
