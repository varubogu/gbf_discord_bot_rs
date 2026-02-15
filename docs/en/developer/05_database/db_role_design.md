# DB Role Design

## Purpose

Split permissions to limit blast radius in case of accidents (accidental deletes, unexpected reads, etc.).

## Principles

- Prepare roles by purpose (e.g., normal operation, admin, cleanup)
- Least privilege (allow only required operations)

## Roles to create and their permissions

### `gbf_bot_admin`

Primarily for migrations.
Has read/write permissions on all tables, effectively close to root (but not a superuser).
Keep usage to a minimum.

### `gbf_bot_global`

For master data shared across all Discord servers.
Has read/write permissions mainly on the `global` schema and is the next-strongest role after admin.
Do not grant access to `guild` schemas.

### `gbf_bot_system`

For system settings. Read/write permissions on system-related tables (e.g., scheduler processing).

- Bot runtime: normal read/write (as needed)
- Admin tasks: management operations such as migrations
- Maintenance (cleanup): limited permissions for deletion targets

### `gbf_bot_guild`

For guild-specific data.
Read/write permissions on guild-related tables.
When using this role, specify `guild_id` on connection and enforce access control via RLS.

### `gbf_bot_cleanup`

For cleanup.
Limited permissions only for required tables.

## Operational notes
