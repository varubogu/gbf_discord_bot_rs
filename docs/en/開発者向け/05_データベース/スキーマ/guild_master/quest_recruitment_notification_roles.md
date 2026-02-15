# guild_master.quest_recruitment_notification_roles Table Design

## Overview

- Schema: `guild_master`
- Table: `quest_recruitment_notification_roles`
- Source: `src/models/entities/guild_master/quest_recruitment_notification_roles.rs`

## Primary key

- guild_id, quest_id, seq

## Columns (code-aligned)

| Column | Type (Rust) | Nullable | Notes |
| --- | --- | --- | --- |
| `guild_id` | `i64` | NO | Primary key |
| `quest_id` | `i32` | NO | Primary key |
| `seq` | `i32` | NO | Primary key |
| `role_id` | `i64` | NO |  |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## Notes

- This document is created using the definitions in `src/models/entities` as the source of truth.
- For final constraints and indexes, also check the migration definitions.
