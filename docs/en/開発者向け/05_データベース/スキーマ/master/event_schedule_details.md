# master.event_schedule_details テーブル設計

## 概要

- スキーマ: `master`
- テーブル: `event_schedule_details`
- 実装ソース: `src/models/entities/master/event_schedule_details.rs`

## 主キー

- id

## カラム定義（コード準拠）

| カラム | 型（Rust） | NULL許容 | 備考 |
| --- | --- | --- | --- |
| `id` | `Uuid` | NO | 主キー |
| `profile` | `String` | NO |  |
| `start_day_relative` | `String` | NO |  |
| `time` | `String` | NO |  |
| `schedule_name` | `String` | NO |  |
| `message_text_id` | `String` | NO |  |
| `notification_channel_type` | `i32` | NO |  |
| `reactions` | `String` | NO |  |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## 補足

- 本書は `src/models/entities` の定義を正として作成しています。
- 制約・インデックスの最終情報はマイグレーション定義も併せて確認してください。
