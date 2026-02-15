# worker.quest_matchings Table Design

## Overview

- Schema: `worker`
- Table: `quest_matchings`
- Source: `src/models/entities/worker/quest_matchings.rs`

## Primary key

- guild_id, id

## Columns (code-aligned)

| Column | Type (Rust) | Nullable | Notes |
| --- | --- | --- | --- |
| `guild_id` | `i64` | NO | Primary key |
| `id` | `Uuid` | NO | Primary key |
| `quest_id` | `i32` | NO |  |
| `scheduled_month` | `i32` | NO |  |
| `scheduled_day` | `i32` | NO |  |
| `scheduled_hour` | `i32` | NO |  |
| `status` | `String` | NO |  |
| `recruitment_id` | `Option<i32>` | YES |  |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## Notes

- This document is created using the definitions in `src/models/entities` as the source of truth.
- For final constraints and indexes, also check the migration definitions.
