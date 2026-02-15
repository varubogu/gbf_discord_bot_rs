# guild_master.guild_event_schedules テーブル設計

## 概要

- スキーマ: `guild_master`
- テーブル: `guild_event_schedules`
- 実装ソース: `src/models/entities/guild_master/guild_event_schedules.rs`

## 主キー

- guild_id, id

## カラム定義（コード準拠）

| カラム | 型（Rust） | NULL許容 | 備考 |
| --- | --- | --- | --- |
| `guild_id` | `i64` | NO | 主キー |
| `id` | `Uuid` | NO | 主キー |
| `event_type` | `String` | NO |  |
| `event_count` | `i64` | NO |  |
| `profile` | `String` | NO |  |
| `weak_attribute` | `i32` | NO |  |
| `start_at` | `DateTime` | NO |  |
| `end_at` | `DateTime` | NO |  |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## 補足

- 本書は `src/models/entities` の定義を正として作成しています。
- 制約・インデックスの最終情報はマイグレーション定義も併せて確認してください。
