# guild_master.auto_recruitment_participants Table Design

## Overview

- Schema: `guild_master`
- Table: `auto_recruitment_participants`
- Source: `src/models/entities/guild_master/auto_recruitment_participants.rs`

## Primary key

- guild_id, user_id, month, day, hour

## Columns (code-aligned)

| Column | Type (Rust) | Nullable | Notes |
| --- | --- | --- | --- |
| `guild_id` | `i64` | NO | Primary key |
| `user_id` | `i64` | NO | Primary key |
| `month` | `i32` | NO | Primary key |
| `day` | `i32` | NO | Primary key |
| `hour` | `i32` | NO | Primary key |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## Notes

- This document is created using the definitions in `src/models/entities` as the source of truth.
- For final constraints and indexes, also check the migration definitions.
