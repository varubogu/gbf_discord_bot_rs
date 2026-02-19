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
| 1 | イベントスケジュール通知 | イベントスケジュール通知の送信先チャンネル |
| 2 | マルチ募集 | マルチ募集メッセージの送信先チャンネル |
| 5 | 管理者通知 | bot実行中のエラーや設定不足を管理者（gbf_bot_controlロール保持者）に通知するチャンネル |

## Rust enum

The IDs in this table are represented as `GuildChannelType` defined in `src/models/entities/master/channel_types.rs`.
See the coding standards for the rule that requires fixed master IDs to be expressed as enums.

## Notes

- This document is created using the definitions in `src/models/entities` as the source of truth.
- For final constraints and indexes, also check the migration definitions.
