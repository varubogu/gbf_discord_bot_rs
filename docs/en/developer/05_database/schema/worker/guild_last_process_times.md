# worker.guild_last_process_times Table Design

## Overview

- Schema: `worker`
- Table: `guild_last_process_times`
- Source: `src/models/entities/worker/guild_last_process_times.rs`

## Primary key

- guild_id, process_type

## Columns (code-aligned)

| Column | Type (Rust) | Nullable | Notes |
| --- | --- | --- | --- |
| `guild_id` | `i64` | NO | Primary key |
| `process_type` | `i32` | NO | Primary key |
| `execute_time` | `Option<DateTimeUtc>` | YES |  |
| `memo` | `String` | NO |  |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## Notes

- This document is created using the definitions in `src/models/entities` as the source of truth.
- For final constraints and indexes, also check the migration definitions.
