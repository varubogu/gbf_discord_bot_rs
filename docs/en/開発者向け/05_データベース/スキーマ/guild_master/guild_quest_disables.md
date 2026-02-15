# guild_master.guild_quest_disables Table Design

## Overview

- Schema: `guild_master`
- Table: `guild_quest_disables`
- Source: `src/models/entities/guild_master/guild_quest_disables.rs`

## Primary key

- guild_id, quest_id

## Columns (code-aligned)

| Column | Type (Rust) | Nullable | Notes |
| --- | --- | --- | --- |
| `guild_id` | `i64` | NO | Primary key |
| `quest_id` | `i32` | NO | Primary key |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## Notes

- This document is created using the definitions in `src/models/entities` as the source of truth.
- For final constraints and indexes, also check the migration definitions.
