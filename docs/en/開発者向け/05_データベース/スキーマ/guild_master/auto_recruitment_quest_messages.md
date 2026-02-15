# guild_master.auto_recruitment_quest_messages Table Design

## Overview

- Schema: `guild_master`
- Table: `auto_recruitment_quest_messages`
- Source: `src/models/entities/guild_master/auto_recruitment_quest_messages.rs`

## Primary key

- guild_id, quest_id

## Columns (code-aligned)

| Column | Type (Rust) | Nullable | Notes |
| --- | --- | --- | --- |
| `guild_id` | `i64` | NO | Primary key |
| `quest_id` | `i32` | NO | Primary key |
| `message_id` | `i64` | NO |  |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## Notes

- This document is created using the definitions in `src/models/entities` as the source of truth.
- For final constraints and indexes, also check the migration definitions.
