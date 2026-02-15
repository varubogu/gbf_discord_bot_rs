# worker.quest_matching_users Table Design

## Overview

- Schema: `worker`
- Table: `quest_matching_users`
- Source: `src/models/entities/worker/quest_matching_users.rs`

## Primary key

- guild_id, matching_id, user_id

## Columns (code-aligned)

| Column | Type (Rust) | Nullable | Notes |
| --- | --- | --- | --- |
| `guild_id` | `i64` | NO | Primary key |
| `matching_id` | `Uuid` | NO | Primary key |
| `user_id` | `i64` | NO | Primary key |
| `battle_style_id` | `Option<i32>` | YES |  |
| `joined_at` | `DateTimeUtc` | NO |  |
| `left_at` | `Option<DateTimeUtc>` | YES |  |

## Notes

- This document is created using the definitions in `src/models/entities` as the source of truth.
- For final constraints and indexes, also check the migration definitions.
