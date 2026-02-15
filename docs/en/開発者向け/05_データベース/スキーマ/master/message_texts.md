# master.message_texts Table Design

## Overview

- Schema: `master`
- Table: `message_texts`
- Source: `src/models/entities/master/message_texts.rs`

## Primary key

- id

## Columns (code-aligned)

| Column | Type (Rust) | Nullable | Notes |
| --- | --- | --- | --- |
| `id` | `String` | NO | Primary key |
| `message_jp` | `String` | NO |  |
| `message_en` | `Option<String>` | YES |  |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## Notes

- This document is created using the definitions in `src/models/entities` as the source of truth.
- For final constraints and indexes, also check the migration definitions.
