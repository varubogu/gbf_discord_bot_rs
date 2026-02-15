# guild_master.guild_channels Table Design

## Overview

- Schema: `guild_master`
- Table: `guild_channels`
- Source: `src/models/entities/guild_master/guild_channels.rs`

## Primary key

- guild_id, channel_type

## Columns (code-aligned)

| Column | Type (Rust) | Nullable | Notes |
| --- | --- | --- | --- |
| `guild_id` | `i64` | NO | Primary key |
| `channel_type` | `i32` | NO | Primary key |
| `channel_id` | `i64` | NO |  |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## Notes

- This document is created using the definitions in `src/models/entities` as the source of truth.
- For final constraints and indexes, also check the migration definitions.
