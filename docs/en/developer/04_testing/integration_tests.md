# Integration Tests

Integration tests verify the consistency of use cases that span multiple layers.
In this project, they are written primarily **from facades**.

## Location

- Under `tests/integration/`
- Example: `tests/integration/facades/` (facade entry point)

## Tests requiring a real DB

Facade integration tests use a real PostgreSQL provided by testcontainers.
The fixture lives in `tests/integration/facades/test_helper.rs`.

- **One container per test binary.** It is started once via `OnceCell` and shared by every test in that binary.
- **One template database per container.** Right after startup the fixture creates the application DB
  roles used by RLS, enables `pgcrypto`, applies all migrations, and seeds the shared master data into
  a template database. The connection used for that setup is closed so the template can be cloned.
- **One database per test.** Every call to `create_test_app_state()` clones the template with
  `CREATE DATABASE ... TEMPLATE ...` and returns an `AppState` bound to that fresh database.
  Cloning a template is a file copy, so it is far cheaper than re-running migrations. Nothing stays
  connected to the template after setup, so clones may run concurrently — do not serialize them,
  that alone made the suite four times slower.

Because each test owns its database, tests are isolated from one another:

- Tests may reuse the same `guild_id` without colliding.
- Unfiltered `delete_many()` in one test cannot affect another test.
- The suite runs in parallel; `RUST_TEST_THREADS` does not need to be pinned to 1.

Costs to be aware of:

- Each clone occupies roughly 10 MB, so a full run holds around 1.5 GB inside the container.
  It is reclaimed when the throwaway container is removed at the end of the run.
- The container raises `max_connections` because every test owns its own connection pools.
  Pools are capped low (`max_connections(4)`, `min_connections(0)`) for the same reason.
- Connection pools must not be cached in a `static`. `#[tokio::test]` builds a runtime per test and
  drops it afterwards, so a pool created during the first test stops working once that test ends.
  The fixture opens a short-lived admin connection per `CREATE DATABASE` instead.

Cleanup within a test is still worth keeping where it documents the precondition, but it is no longer
what provides isolation.

`#[ignore]` is reserved for tests that require an external secret or external API; it must not be used
solely because a test needs PostgreSQL.

### Taking more than one connection to the same database

`create_test_app_state()` creates a **new** database on every call, so calling it twice in one test
yields two unrelated databases. When a test needs several connections — for example a facade plus a
role-specific connection for verifying RLS — obtain a `TestDb` handle first and derive every
connection from it:

```rust
let test_db = TestDb::new().await;
let app_state = Arc::new(test_db.app_state().await);
let guild_role_db = test_db.guild_role_db().await; // same database as app_state
```

| Method | Role | Purpose |
| --- | --- | --- |
| `TestDb::app_state()` | `postgres` | `AppState` for facade calls (RLS bypassed) |
| `TestDb::guild_db()` | `postgres` | Direct connection for arranging/asserting data |
| `TestDb::guild_role_db()` | `gbf_bot_guild` | Verifying RLS (`set_current_guild_id`) behavior |
| `TestDb::admin_db()` | `gbf_bot_admin` | Operations that require the admin role |

## CI execution

- The regular test lane runs facade integration tests without a PostgreSQL service container or DB credentials.
- The runner must permit Docker access so the test fixture can start its PostgreSQL container.
- External-secret tests remain ignored and are executed only in their dedicated workflow.

## Test data handling

- Prefer dedicated test `guild_id` / `user_id` values that describe what the test is about
- Delete target data explicitly when the test depends on a clean starting state

## Design notes

- One test should have one purpose
- Write assertions so a failure clearly shows what broke

## Per-feature design docs (recommended)

- [integration_test_design](integration_test_design/README.md)
