# worker.scheduled_tasks Table Design

## Overview

- Schema: `worker`
- Table: `scheduled_tasks`
- Source: `src/models/entities/worker/scheduled_tasks.rs` (sync after implementation update)

## Primary key

- id

## Columns (design)

| Column | Type (DB) | Nullable | Notes |
| --- | --- | --- | --- |
| `id` | `serial` | NO | Primary key |
| `schedule_datetime` | `timestamptz` | NO |  |
| `task_type` | `int` | NO |  |
| `guild_id` | `bigint` | YES |  |
| `channel_id` | `bigint` | YES |  |
| `execution_status` | `worker.task_execution_status` | NO | Execution status (default: `pending`) |
| `created_at` | `timestamptz` | NO |  |
| `updated_at` | `timestamptz` | NO |  |

## ENUM definition

### `worker.task_execution_status`

| Value | Meaning | Included in next scheduler run |
| --- | --- | --- |
| `pending` | Not executed yet | Yes |
| `succeeded` | Completed successfully | No |
| `succeeded_with_warning` | Completed successfully with warning(s) | No |
| `failed` | Completed with error | No |

## Index policy (excerpt)

- The partial index for fetching unexecuted tasks should use `execution_status = 'pending'`

## Notes

- This document describes the new design that introduces ENUM-based execution status.
- For final constraints and indexes, also check the migration definitions.
