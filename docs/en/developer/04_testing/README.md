# Testing (Developer)

This section organizes “what to test” and “at what granularity”.

## Common commands

```bash
# Run a focused test (filter by name)
cargo test -j 1 test_name

# Full suite
cargo test -j 1

# Also run ignored tests (e.g., those requiring a real DB)
cargo test -j 1 -- --ignored
```

## Start here

1. [Overall Test Design](test_overview.md)
2. [Unit Tests](unit_tests.md)
3. [Integration Tests](integration_tests.md)
4. [Ignored Test Execution Strategy](ignored_test_strategy.md)
