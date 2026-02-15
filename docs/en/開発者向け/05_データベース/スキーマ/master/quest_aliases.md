# master.quest_aliases Table Design

## Overview

- Schema: `master`
- Table: `quest_aliases`
- Source: `src/models/entities/master/quest_aliases.rs`

## Primary key

- quest_id, sequence_no

## Columns (code-aligned)

| Column | Type (Rust) | Nullable | Notes |
| --- | --- | --- | --- |
| `quest_id` | `i32` | NO | Primary key |
| `sequence_no` | `i32` | NO | Primary key |
| `alias` | `String` | NO |  |
| `alias_kana_small` | `String` | NO |  |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## Notes

- This document is created using the definitions in `src/models/entities` as the source of truth.
- For final constraints and indexes, also check the migration definitions.
