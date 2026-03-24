# Migrations

## Purpose

Safely update DB structure (tables/columns/permissions).

## Run (dev/ops)

```bash
cargo run -j 1 -- migrate
```

## Create a new migration (development)

```bash
cd migration
sea-orm-cli migrate generate migration_name
```

## Notes

- Design changes to be safely applicable with existing production data in place
- Migration changes can affect DB permissions (roles), so verify the impact scope before applying
