# master.battle_styles Table Design

## Overview

- Schema: `master`
- Table: `battle_styles`
- Source: `src/models/entities/master/battle_styles.rs`

## Primary key

- id

## Columns (code-aligned)

| Column | Type (Rust) | Nullable | Notes |
| --- | --- | --- | --- |
| `id` | `i32` | NO | Primary key |
| `display_name` | `String` | NO |  |
| `reactions` | `Option<String>` | YES |  |
| `sort_order` | `i32` | NO |  |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## Notes

- This document is created using the definitions in `src/models/entities` as the source of truth.
- For final constraints and indexes, also check the migration definitions.
