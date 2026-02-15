# master.event_schedule_details Table Design

## Overview

- Schema: `master`
- Table: `event_schedule_details`
- Source: `src/models/entities/master/event_schedule_details.rs`

## Primary key

- id

## Columns (code-aligned)

| Column | Type (Rust) | Nullable | Notes |
| --- | --- | --- | --- |
| `id` | `Uuid` | NO | Primary key |
| `profile` | `String` | NO |  |
| `start_day_relative` | `String` | NO |  |
| `time` | `String` | NO |  |
| `schedule_name` | `String` | NO |  |
| `message_text_id` | `String` | NO |  |
| `notification_channel_type` | `i32` | NO |  |
| `reactions` | `String` | NO |  |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## Notes

- This document is created using the definitions in `src/models/entities` as the source of truth.
- For final constraints and indexes, also check the migration definitions.
