# worker.recruitment_participants Table Design

## Overview

- Schema: `worker`
- Table: `recruitment_participants`
- Source: `src/models/entities/worker/recruitment_participants.rs`

## Primary key

- id

## Columns (code-aligned)

| Column | Type (Rust) | Nullable | Notes |
| --- | --- | --- | --- |
| `id` | `i64` | NO | Primary key |
| `recruitment_id` | `i32` | NO |  |
| `user_id` | `i64` | NO |  |
| `element_id` | `Option<i32>` | YES |  |
| `participated_at` | `DateTimeUtc` | NO |  |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## Notes

- This document is created using the definitions in `src/models/entities` as the source of truth.
- For final constraints and indexes, also check the migration definitions.
