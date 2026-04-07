# guild_master.auto_recruitment_match_rules Table Design

## Overview

- Schema: `guild_master`
- Table: `auto_recruitment_match_rules`
- Source: `src/models/entities/guild_master/auto_recruitment_match_rules.rs`

## Primary key

- guild_id, quest_id

## Columns (code-aligned)

| Column | Type (Rust) | Nullable | Notes |
| --- | --- | --- | --- |
| `guild_id` | `i64` | NO | Primary key (`0` means global scope) |
| `quest_id` | `i32` | NO | Primary key |
| `preset_type` | `String` | NO | Matching preset name |
| `min_match_count` | `i32` | NO | Minimum users required |
| `required_battle_style_id` | `Option<i32>` | YES | Used only by `specific_element_n_plus_any` |
| `required_battle_style_count` | `Option<i32>` | YES | Used only by `specific_element_n_plus_any` |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## Notes

- This document is created using the definitions in `src/models/entities` as the source of truth.
- For final constraints and indexes, also check the migration definitions.
- Runtime resolution priority is `guild-specific (guild_id=<target>) -> global (guild_id=0) -> no rule`.
