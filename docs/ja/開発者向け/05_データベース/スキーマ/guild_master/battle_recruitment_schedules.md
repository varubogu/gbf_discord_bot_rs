# guild_master.battle_recruitment_schedules テーブル設計

## 概要

- スキーマ: `guild_master`
- テーブル: `battle_recruitment_schedules`
- 実装ソース: `src/models/entities/guild_master/battle_recruitment_schedules.rs`

## 主キー

- id

## カラム定義（コード準拠）

| カラム | 型（Rust） | NULL許容 | 備考 |
| --- | --- | --- | --- |
| `id` | `i32` | NO | 主キー |
| `name` | `String` | NO |  |
| `guild_id` | `i64` | NO |  |
| `channel_id` | `i64` | NO |  |
| `quest_id` | `i32` | NO |  |
| `battle_style_id` | `i32` | NO |  |
| `quest_start_time` | `TimeTime` | NO |  |
| `recruit_start_day_offset` | `i32` | NO |  |
| `recruit_start_time` | `Option<TimeTime>` | YES |  |
| `max_participants` | `Option<i32>` | YES |  |
| `note` | `Option<String>` | YES |  |
| `is_enabled` | `bool` | NO |  |
| `created_by` | `i64` | NO |  |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## 補足

- 本書は `src/models/entities` の定義を正として作成しています。
- 制約・インデックスの最終情報はマイグレーション定義も併せて確認してください。
