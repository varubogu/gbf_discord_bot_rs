# master.elements Table Design

## Overview

- Schema: `master`
- Table: `elements`
- Source: `src/models/entities/master/elements.rs`

## Primary key

- id

## Columns (code-aligned)

| Column | Type (Rust) | Nullable | Notes |
| --- | --- | --- | --- |
| `id` | `i32` | NO | Primary key |
| `reaction_stamp` | `Option<String>` | YES |  |
| `name_jp` | `String` | NO |  |
| `name_en` | `Option<String>` | YES |  |

## Notes

- This document is created using the definitions in `src/models/entities` as the source of truth.
- For final constraints and indexes, also check the migration definitions.
