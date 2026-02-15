# worker.battle_recruitment_dismissals Table Design

## Overview

- Schema: `worker`
- Table: `battle_recruitment_dismissals`
- Source: `src/models/entities/worker/battle_recruitment_dismissals.rs`

## Primary key

- id

## Columns (code-aligned)

| Column | Type (Rust) | Nullable | Notes |
| --- | --- | --- | --- |
| `id` | `i32` | NO | Primary key |
| `recruitment_id` | `i32` | NO |  |
| `input_value` | `String` | NO |  |
| `input_type` | `i32` | NO |  |
| `dismissal_datetime` | `Option<DateTimeUtc>` | YES |  |
| `relative_days` | `Option<i32>` | YES |  |
| `relative_hours` | `Option<i32>` | YES |  |
| `relative_minutes` | `Option<i32>` | YES |  |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## Notes

- This document is created using the definitions in `src/models/entities` as the source of truth.
- For final constraints and indexes, also check the migration definitions.
