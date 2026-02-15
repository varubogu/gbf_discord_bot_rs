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

1. [テスト全体設計](テスト全体設計.md)
2. [単体テスト](単体テスト.md)
3. [結合テスト](結合テスト.md)
