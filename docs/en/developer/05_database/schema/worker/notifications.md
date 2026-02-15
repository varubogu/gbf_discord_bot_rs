# worker.notifications Table Design

## Overview

- Schema: `worker`
- Table: `notifications`
- Source: `src/models/entities/worker/notifications.rs`

## Primary key

- id

## Columns (code-aligned)

| Column | Type (Rust) | Nullable | Notes |
| --- | --- | --- | --- |
| `id` | `i32` | NO | Primary key |
| `task_id` | `i32` | NO |  |
| `guild_id` | `i64` | NO |  |
| `channel_id` | `i64` | NO |  |
| `message_text_id` | `String` | NO |  |
| `is_sent` | `bool` | NO |  |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## Notes

- This document is created using the definitions in `src/models/entities` as the source of truth.
- For final constraints and indexes, also check the migration definitions.
