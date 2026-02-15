# worker.scheduled_task_cleanups Table Design

## Overview

- Schema: `worker`
- Table: `scheduled_task_cleanups`
- Source: `src/models/entities/worker/scheduled_task_cleanups.rs`

## Primary key

- task_id

## Columns (code-aligned)

| Column | Type (Rust) | Nullable | Notes |
| --- | --- | --- | --- |
| `task_id` | `i32` | NO | Primary key |
| `target_schema` | `String` | NO |  |
| `target_table` | `String` | NO |  |
| `cleanup_before` | `DateTimeUtc` | NO |  |

## Notes

- This document is created using the definitions in `src/models/entities` as the source of truth.
- For final constraints and indexes, also check the migration definitions.
