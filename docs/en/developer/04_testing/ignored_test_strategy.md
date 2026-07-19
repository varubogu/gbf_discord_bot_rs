# Ignored Test Execution Strategy

Last updated: 2026-03-24

## Purpose

- Classify the large number of `#[ignore]` tests and separate them into dedicated execution lanes
- Keep important but heavy regression checks running continuously in CI

## Classification

| Category | Representative paths | Execution lane |
|---|---|---|
| Facade integration tests | `tests/integration/facades/*` | Regular test lane; PostgreSQL is supplied by the shared test container |
| External API or token required | `tests/system/bot_startup_test.rs` | Manual execution only |
| Standalone maintenance DB tests | `tests/data_cleanup_test.rs` | Dedicated DB test lane until they use the shared fixture |

## Operational rules

1. If a test uses `#[ignore]`, add the reason on the same line as a comment.
2. A facade integration test must not use `#[ignore]` merely because it requires PostgreSQL.
3. Tests that require Discord tokens or other external secrets must stay out of the regular CI lane and be limited to manual execution.
4. Standalone DB tests are migrated to the shared fixture before they are promoted to the regular lane.

## CI lanes

- Regular lane: `.github/workflows/ci.yml`
  - `cargo fmt -- --check`
  - `cargo clippy -j 1`
  - `cargo test -j 1`
- The regular lane starts a PostgreSQL test container for facade integration tests.
- The runner must allow Docker access; no application DB credentials or PostgreSQL service are required.
- The dedicated DB workflow remains for standalone maintenance tests until their fixture migration is complete.

## Local execution examples

```bash
# Equivalent to the regular lane
cargo fmt -- --check
cargo clippy -j 1
cargo test -j 1

# Facade integration tests
cargo test -j 1 --test mod
```
