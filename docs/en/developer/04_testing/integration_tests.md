# Integration Tests

Integration tests verify the consistency of use cases that span multiple layers.
In this project, they are written primarily **from facades**.

## Location

- Under `tests/integration/`
- Example: `tests/integration/facades/` (facade entry point)

## Tests requiring a real DB

- Facade integration tests start one PostgreSQL container per test binary and apply all migrations before use.
- The fixture creates the application DB roles used by RLS, so tests must use `AppState` from the shared fixture rather than a local connection or environment variables.
- Each test remains responsible for using distinct test data and cleaning up data it creates.
- `#[ignore]` is reserved for tests that require an external secret or external API; it must not be used solely because a test needs PostgreSQL.

## CI execution

- The regular test lane runs facade integration tests without a PostgreSQL service container or DB credentials.
- The runner must permit Docker access so the test fixture can start its PostgreSQL container.
- External-secret tests remain ignored and are executed only in their dedicated workflow.

## Test data handling

- Use dedicated test `guild_id` / `user_id` values so tests do not collide with each other
- Delete target data explicitly before each test and after each test so reruns stay safe

## Design notes

- One test should have one purpose
- Write assertions so a failure clearly shows what broke

## Per-feature design docs (recommended)

- [integration_test_design](integration_test_design/README.md)
