# guild_master.guild_event_schedules Table Design

## Overview

- Schema: `guild_master`
- Table: `guild_event_schedules`
- Source: `src/models/entities/guild_master/guild_event_schedules.rs`

## Primary key

- guild_id, id

## Columns (code-aligned)

| Column | Type (Rust) | Nullable | Notes |
| --- | --- | --- | --- |
| `guild_id` | `i64` | NO | Primary key |
| `id` | `Uuid` | NO | Primary key |
| `event_type` | `String` | NO |  |
| `event_count` | `i64` | NO |  |
| `profile` | `String` | NO |  |
| `weak_attribute` | `i32` | NO |  |
| `start_at` | `DateTime` | NO |  |
| `end_at` | `DateTime` | NO |  |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## Notes

- This document is created using the definitions in `src/models/entities` as the source of truth.
- For final constraints and indexes, also check the migration definitions.
