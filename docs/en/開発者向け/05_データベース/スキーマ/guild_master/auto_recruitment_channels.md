# guild_master.auto_recruitment_channels Table Design

## Overview

- Schema: `guild_master`
- Table: `auto_recruitment_channels`
- Source: `src/models/entities/guild_master/auto_recruitment_channels.rs`

## Primary key

- id

## Columns (code-aligned)

| Column | Type (Rust) | Nullable | Notes |
| --- | --- | --- | --- |
| `id` | `i32` | NO | Primary key |
| `guild_id` | `i64` | NO |  |
| `channel_id` | `i64` | NO |  |
| `month` | `i32` | NO |  |
| `day` | `i32` | NO |  |
| `sort_order` | `i32` | NO |  |
| `is_bot_created` | `bool` | NO |  |
| `message_id` | `Option<i64>` | YES |  |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## Notes

- This document is created using the definitions in `src/models/entities` as the source of truth.
- For final constraints and indexes, also check the migration definitions.
