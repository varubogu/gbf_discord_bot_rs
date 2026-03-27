# guild_master.auto_recruitment_match_rule_quotas Table Design

## Overview

- Schema: `guild_master`
- Table: `auto_recruitment_match_rule_quotas`
- Source: `src/models/entities/guild_master/auto_recruitment_match_rule_quotas.rs`

## Primary key

- guild_id, quest_id, battle_style_id

## Columns (code-aligned)

| Column | Type (Rust) | Nullable | Notes |
| --- | --- | --- | --- |
| `guild_id` | `i64` | NO | Primary key |
| `quest_id` | `i32` | NO | Primary key |
| `battle_style_id` | `i32` | NO | Primary key |
| `required_count` | `i32` | NO | Required users for this style |
| `sort_order` | `i32` | NO | Stable display and processing order |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## Notes

- This document is created using the definitions in `src/models/entities` as the source of truth.
- For final constraints and indexes, also check the migration definitions.
