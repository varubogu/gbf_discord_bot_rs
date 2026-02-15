# guild_master.user_desired_quests Table Design

## Overview

- Schema: `guild_master`
- Table: `user_desired_quests`
- Source: `src/models/entities/guild_master/user_desired_quests.rs`

## Primary key

- guild_id, user_id, quest_id, battle_style_id

## Columns (code-aligned)

| Column | Type (Rust) | Nullable | Notes |
| --- | --- | --- | --- |
| `guild_id` | `i64` | NO | Primary key |
| `user_id` | `i64` | NO | Primary key |
| `quest_id` | `i32` | NO | Primary key |
| `battle_style_id` | `i32` | NO | Primary key |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## Notes

- This document is created using the definitions in `src/models/entities` as the source of truth.
- For final constraints and indexes, also check the migration definitions.
