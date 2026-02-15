# worker.scheduled_tasks Table Design

## Overview

- Schema: `worker`
- Table: `scheduled_tasks`
- Source: `src/models/entities/worker/scheduled_tasks.rs`

## Primary key

- id

## Columns (code-aligned)

| Column | Type (Rust) | Nullable | Notes |
| --- | --- | --- | --- |
| `id` | `i32` | NO | Primary key |
| `schedule_datetime` | `DateTimeUtc` | NO |  |
| `task_type` | `i32` | NO |  |
| `guild_id` | `Option<i64>` | YES |  |
| `channel_id` | `Option<i64>` | YES |  |
| `is_executed` | `bool` | NO |  |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## Notes

- This document is created using the definitions in `src/models/entities` as the source of truth.
- For final constraints and indexes, also check the migration definitions.
