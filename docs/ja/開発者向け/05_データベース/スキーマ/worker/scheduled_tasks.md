# worker.scheduled_tasks テーブル設計

## 概要

- スキーマ: `worker`
- テーブル: `scheduled_tasks`
- 実装ソース: `src/models/entities/worker/scheduled_tasks.rs`

## 主キー

- id

## カラム定義（コード準拠）

| カラム | 型（Rust） | NULL許容 | 備考 |
| --- | --- | --- | --- |
| `id` | `i32` | NO | 主キー |
| `schedule_datetime` | `DateTimeUtc` | NO |  |
| `task_type` | `i32` | NO |  |
| `guild_id` | `Option<i64>` | YES |  |
| `channel_id` | `Option<i64>` | YES |  |
| `is_executed` | `bool` | NO |  |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## 補足

- 本書は `src/models/entities` の定義を正として作成しています。
- 制約・インデックスの最終情報はマイグレーション定義も併せて確認してください。
