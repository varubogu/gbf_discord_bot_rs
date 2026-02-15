# worker.scheduled_task_recurring_recruitments Table Design

## Overview

- Schema: `worker`
- Table: `scheduled_task_recurring_recruitments`
- Source: `src/models/entities/worker/scheduled_task_recurring_recruitments.rs`

## Primary key

- scheduled_task_id

## Columns (code-aligned)

| Column | Type (Rust) | Nullable | Notes |
| --- | --- | --- | --- |
| `scheduled_task_id` | `i32` | NO | Primary key |
| `recruitment_schedule_id` | `i32` | NO |  |

## Notes

- This document is created using the definitions in `src/models/entities` as the source of truth.
- For final constraints and indexes, also check the migration definitions.
