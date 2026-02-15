# guild_master.guilds Table Design

## Overview

- Schema: `guild_master`
- Table: `guilds`
- Source: `src/models/entities/guild_master/guilds.rs`

## Primary key

- guild_id

## Columns (code-aligned)

| Column | Type (Rust) | Nullable | Notes |
| --- | --- | --- | --- |
| `guild_id` | `i64` | NO | Primary key |
| `name` | `String` | NO |  |
| `recruit_channel_id` | `Option<i64>` | YES |  |
| `timezone` | `Option<String>` | YES |  |
| `default_recruit_duration` | `Option<i32>` | YES |  |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## Notes

- This document is created using the definitions in `src/models/entities` as the source of truth.
- For final constraints and indexes, also check the migration definitions.
