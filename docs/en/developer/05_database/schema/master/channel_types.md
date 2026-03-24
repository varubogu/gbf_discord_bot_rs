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

## Master data values

| id | name | memo |
| --- | --- | --- |
| 1 | Event schedule notification | Destination channel for event schedule notifications |
| 2 | Co-op recruitment | Destination channel for co-op recruitment messages |
| 3 | Guild contact | Destination channel for guild notices (if members-only, also grant bot access) |
| 4 | Shared co-op recruitment (cross-server) | Destination used when recruiting from an external guild; may reuse the normal co-op channel or remain undefined |
| 5 | Admin notification | Channel used to notify administrators (users with `gbf_bot_control`) about runtime errors and missing settings |

## Rust enum

The IDs in this table are represented as `GuildChannelType` defined in `src/models/entities/master/channel_types.rs`.
See the coding standards for the rule that requires fixed master IDs to be expressed as enums.

## Notes

- This document is created using the definitions in `src/models/entities` as the source of truth.
- For final constraints and indexes, also check the migration definitions.
