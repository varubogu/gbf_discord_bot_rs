# guild_master.guild_spreadsheet_exports Table Design

## Overview

- Schema: `guild_master`
- Table: `guild_spreadsheet_exports`
- Source: `src/models/entities/guild_master/guild_spreadsheet_exports.rs`

## Primary key

- guild_id

## Columns (code-aligned)

| Column | Type (Rust) | Nullable | Notes |
| --- | --- | --- | --- |
| `guild_id` | `i64` | NO | Primary key |
| `spreadsheet_id` | `String` | NO |  |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## Notes

- This document is created using the definitions in `src/models/entities` as the source of truth.
- For final constraints and indexes, also check the migration definitions.
