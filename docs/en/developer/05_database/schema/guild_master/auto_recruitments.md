# guild_master.auto_recruitments Table Design

## Overview

- Schema: `guild_master`
- Table: `auto_recruitments`
- Source: `src/models/entities/guild_master/auto_recruitments.rs`

## Primary key

- guild_id

## Columns (code-aligned)

| Column | Type (Rust) | Nullable | Notes |
| --- | --- | --- | --- |
| `guild_id` | `i64` | NO | Primary key |
| `category_id` | `i64` | NO |  |
| `matching_channel_id` | `Option<i64>` | YES |  |
| `quest_channel_id` | `Option<i64>` | YES |  |
| `matching_channel_is_bot_created` | `bool` | NO |  |
| `quest_channel_is_bot_created` | `bool` | NO |  |
| `matching_message_id` | `Option<i64>` | YES |  |
| `days_range` | `i32` | NO |  |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## Notes

- This document is created using the definitions in `src/models/entities` as the source of truth.
- For final constraints and indexes, also check the migration definitions.
