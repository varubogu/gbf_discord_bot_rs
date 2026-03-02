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

1. [テスト全体設計](test_overview.md)
2. [単体テスト](unit_tests.md)
3. [結合テスト](integration_tests.md)
4. [ignoredテスト実行戦略](ignored_test_strategy.md)
