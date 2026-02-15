# worker.battle_recruitments Table Design

## Overview

- Schema: `worker`
- Table: `battle_recruitments`
- Source: `src/models/entities/worker/battle_recruitments.rs`

## Primary key

- id

## Columns (code-aligned)

| Column | Type (Rust) | Nullable | Notes |
| --- | --- | --- | --- |
| `id` | `i32` | NO | Primary key |
| `guild_id` | `i64` | NO |  |
| `channel_id` | `i64` | NO |  |
| `message_id` | `i64` | NO |  |
| `quest_id` | `i32` | NO |  |
| `battle_style_id` | `i32` | NO |  |
| `quest_start_at` | `DateTimeUtc` | NO |  |
| `is_recruiting` | `bool` | NO |  |
| `is_canceled` | `bool` | NO |  |
| `recruit_end_message_id` | `Option<i64>` | YES |  |
| `full_notification_sent` | `bool` | NO |  |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## Notes

- This document is created using the definitions in `src/models/entities` as the source of truth.
- For final constraints and indexes, also check the migration definitions.
