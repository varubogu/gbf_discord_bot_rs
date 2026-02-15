# worker.notification_rel_battle_recruitments Table Design

## Overview

- Schema: `worker`
- Table: `notification_rel_battle_recruitments`
- Source: `src/models/entities/worker/notification_rel_battle_recruitments.rs`

## Primary key

- recruit_id, notification_id

## Columns (code-aligned)

| Column | Type (Rust) | Nullable | Notes |
| --- | --- | --- | --- |
| `recruit_id` | `i32` | NO | Primary key |
| `notification_id` | `i32` | NO | Primary key |
| `created_at` | `DateTimeUtc` | NO |  |

## Notes

- This document is created using the definitions in `src/models/entities` as the source of truth.
- For final constraints and indexes, also check the migration definitions.
