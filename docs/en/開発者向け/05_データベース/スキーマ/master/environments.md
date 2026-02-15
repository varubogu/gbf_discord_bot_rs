# master.environments Table Design

## Overview

- Schema: `master`
- Table: `environments`
- Source: `src/models/entities/master/environments.rs`

## Primary key

- key

## Columns (code-aligned)

| Column | Type (Rust) | Nullable | Notes |
| --- | --- | --- | --- |
| `key` | `String` | NO | Primary key |
| `value` | `String` | NO |  |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## Notes

- This document is created using the definitions in `src/models/entities` as the source of truth.
- For final constraints and indexes, also check the migration definitions.
