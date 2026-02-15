# master.channel_types Table Design

## Overview

- Schema: `master`
- Table: `channel_types`
- Source: `src/models/entities/master/channel_types.rs`

## Primary key

- id

## Columns (code-aligned)

| Column | Type (Rust) | Nullable | Notes |
| --- | --- | --- | --- |
| `id` | `i32` | NO | Primary key |
| `name` | `String` | NO |  |
| `memo` | `Option<String>` | YES |  |

## Notes

- This document is created using the definitions in `src/models/entities` as the source of truth.
- For final constraints and indexes, also check the migration definitions.
