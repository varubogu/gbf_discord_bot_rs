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

- 既存データがある前提で、安全に適用できる変更にする
- マイグレーションには「DB権限（ロール）」が絡むことがあるため、変更の影響範囲を確認する
