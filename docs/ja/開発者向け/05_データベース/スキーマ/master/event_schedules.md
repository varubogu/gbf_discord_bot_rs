# master.event_schedules テーブル設計

## 概要

- スキーマ: `master`
- テーブル: `event_schedules`
- 実装ソース: `src/models/entities/master/event_schedules.rs`

## 主キー

- id

## カラム定義（コード準拠）

| カラム | 型（Rust） | NULL許容 | 備考 |
| --- | --- | --- | --- |
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
