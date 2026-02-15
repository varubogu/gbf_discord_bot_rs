# Database (Developer)

This section covers DB design, migrations, transactions, and schema operations.

## Common commands

```bash
# Normal start (runs migrations too)
cargo run -j 1

# Run migrations only
cargo run -j 1 -- migrate

# Schema consistency check
cargo run -j 1 --bin schema_lint
```

## Start here

1. [概要](概要.md)
2. [接続とトランザクション](接続とトランザクション.md)
3. [マイグレーション](マイグレーション.md)
