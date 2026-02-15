# guild_master.guild_settings Table Design

## Overview

- Schema: `guild_master`
- Table: `guild_settings`
- Source: `src/models/entities/guild_master/guild_settings.rs`

## Primary key

- guild_id

## Columns (code-aligned)

| Column | Type (Rust) | Nullable | Notes |
| --- | --- | --- | --- |
| `guild_id` | `i64` | NO | Primary key |
| `timezone` | `String` | NO |  |
| `locale` | `String` | NO |  |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## Notes

- This document is created using the definitions in `src/models/entities` as the source of truth.
- For final constraints and indexes, also check the migration definitions.
