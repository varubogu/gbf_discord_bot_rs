# guild_master.guild_event_schedule_details テーブル設計

## 概要

- スキーマ: `guild_master`
- テーブル: `guild_event_schedule_details`
- 実装ソース: `src/models/entities/guild_master/guild_event_schedule_details.rs`

## 主キー

- guild_id, id

## カラム定義（コード準拠）

| カラム | 型（Rust） | NULL許容 | 備考 |
| --- | --- | --- | --- |
| `guild_id` | `i64` | NO | 主キー |
| `id` | `Uuid` | NO | 主キー |
| `profile` | `String` | NO |  |
| `start_day_relative` | `String` | NO |  |
| `time` | `String` | NO |  |
| `schedule_name` | `String` | NO |  |
| `message_text_id` | `String` | NO |  |
| `notification_channel_type` | `i32` | NO |  |
| `notification_channel_id` | `Option<i64>` | YES |  |
| `reactions` | `String` | NO |  |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## 補足

- 本書は `src/models/entities` の定義を正として作成しています。
- 制約・インデックスの最終情報はマイグレーション定義も併せて確認してください。
