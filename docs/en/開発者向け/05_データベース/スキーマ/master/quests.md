# master.quests Table Design

## Overview

- Schema: `master`
- Table: `quests`
- Source: `src/models/entities/master/quests.rs`

## Primary key

- id

## Columns (code-aligned)

| Column | Type (Rust) | Nullable | Notes |
| --- | --- | --- | --- |
| `id` | `i32` | NO | Primary key |
| `name` | `String` | NO |  |
| `default_battle_style_id` | `i32` | NO |  |
| `recruit_count` | `i32` | NO |  |
| `available_battle_style_ids` | `String` | NO |  |
| `sort_order` | `i32` | NO |  |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## Notes

- This document is created using the definitions in `src/models/entities` as the source of truth.
- For final constraints and indexes, also check the migration definitions.
