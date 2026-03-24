# Integration Tests

Integration tests verify the consistency of use cases that span multiple layers.
In this project, they are written primarily **from facades**.

## Location

- Under `tests/integration/`
- Example: `tests/integration/facades/` (facade entry point)

## Tests requiring a real DB

- Mark tests that require a real DB with `#[ignore]` so they do not run in the default test suite
- To run real-DB ignored tests explicitly, use `cargo test -- --ignored`
- For ignored test classification and CI lanes, see [ignored_test_strategy.md](ignored_test_strategy.md)
- Test DB connections must use role-specific environment variables (`SYSTEM_DB_*`, `GUILD_DB_*`, `GLOBAL_DB_*`, `ADMIN_DB_*`) instead of the default `DB_USER` / `DB_PASSWORD`

## CI execution lane for ignored tests

- Workflow: `.github/workflows/ignored-db-tests.yml`
- Representative facade integration tests:
  - `integration::facades::spreadsheet_test`
  - `integration::facades::guild_settings_test`

## Test data handling

- Use dedicated test `guild_id` / `user_id` values so tests do not collide with each other
- Delete target data explicitly before each test and after each test so reruns stay safe

## Design notes

- One test should have one purpose
- Write assertions so a failure clearly shows what broke

## Per-feature design docs (recommended)

- [integration_test_design](integration_test_design/README.md)
