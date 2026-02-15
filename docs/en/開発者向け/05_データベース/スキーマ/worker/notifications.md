# worker.notifications テーブル設計

## 概要

- スキーマ: `worker`
- テーブル: `notifications`
- 実装ソース: `src/models/entities/worker/notifications.rs`

## 主キー

- id

## カラム定義（コード準拠）

| カラム | 型（Rust） | NULL許容 | 備考 |
| --- | --- | --- | --- |
| `id` | `i32` | NO | 主キー |
| `task_id` | `i32` | NO |  |
| `guild_id` | `i64` | NO |  |
| `channel_id` | `i64` | NO |  |
| `message_text_id` | `String` | NO |  |
| `is_sent` | `bool` | NO |  |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## 補足

- 本書は `src/models/entities` の定義を正として作成しています。
- 制約・インデックスの最終情報はマイグレーション定義も併せて確認してください。
