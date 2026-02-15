# worker.notification_rel_event_schedules Table Design

## Overview

- Schema: `worker`
- Table: `notification_rel_event_schedules`
- Source: `src/models/entities/worker/notification_rel_event_schedules.rs`

## Primary key

- event_schedule_id, notification_id

## Columns (code-aligned)

| Column | Type (Rust) | Nullable | Notes |
| --- | --- | --- | --- |
| `event_schedule_id` | `Uuid` | NO | Primary key |
| `event_schedule_detail_id` | `Option<Uuid>` | YES |  |
| `notification_id` | `i32` | NO | Primary key |
| `created_at` | `DateTimeUtc` | NO |  |

## Notes

- This document is created using the definitions in `src/models/entities` as the source of truth.
- For final constraints and indexes, also check the migration definitions.
