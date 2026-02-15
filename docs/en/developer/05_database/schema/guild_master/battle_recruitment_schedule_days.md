# guild_master.battle_recruitment_schedule_days Table Design

## Overview

- Schema: `guild_master`
- Table: `battle_recruitment_schedule_days`
- Source: `src/models/entities/guild_master/battle_recruitment_schedule_days.rs`

## Primary key

- id

## Columns (code-aligned)

| Column | Type (Rust) | Nullable | Notes |
| --- | --- | --- | --- |
| `id` | `i32` | NO | Primary key |
| `schedule_id` | `i32` | NO |  |
| `day_of_week` | `i32` | NO |  |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## Notes

- This document is created using the definitions in `src/models/entities` as the source of truth.
- For final constraints and indexes, also check the migration definitions.
