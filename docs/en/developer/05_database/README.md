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

1. [Overview](overview.md)
2. [Connections and transactions](connections_and_transactions.md)
3. [Migrations](migrations.md)
