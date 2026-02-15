# guild_master.battle_recruitment_schedules Table Design

## Overview

- Schema: `guild_master`
- Table: `battle_recruitment_schedules`
- Source: `src/models/entities/guild_master/battle_recruitment_schedules.rs`

## Primary key

- id

## Columns (code-aligned)

| Column | Type (Rust) | Nullable | Notes |
| --- | --- | --- | --- |
| `id` | `i32` | NO | Primary key |
| `name` | `String` | NO |  |
| `guild_id` | `i64` | NO |  |
| `channel_id` | `i64` | NO |  |
| `quest_id` | `i32` | NO |  |
| `battle_style_id` | `i32` | NO |  |
| `quest_start_time` | `TimeTime` | NO |  |
| `recruit_start_day_offset` | `i32` | NO |  |
| `recruit_start_time` | `Option<TimeTime>` | YES |  |
| `max_participants` | `Option<i32>` | YES |  |
| `note` | `Option<String>` | YES |  |
| `is_enabled` | `bool` | NO |  |
| `created_by` | `i64` | NO |  |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## Notes

- This document is created using the definitions in `src/models/entities` as the source of truth.
- For final constraints and indexes, also check the migration definitions.
