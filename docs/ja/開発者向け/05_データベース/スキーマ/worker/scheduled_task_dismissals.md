# worker.scheduled_task_dismissals テーブル設計

## 概要

- スキーマ: `worker`
- テーブル: `scheduled_task_dismissals`
- 実装ソース: `src/models/entities/worker/scheduled_task_dismissals.rs`

## 主キー

- task_id

## カラム定義（コード準拠）

| カラム | 型（Rust） | NULL許容 | 備考 |
| --- | --- | --- | --- |
| `task_id` | `i32` | NO | 主キー |
| `recruitment_dismissal_id` | `i32` | NO |  |
| `created_at` | `DateTimeUtc` | NO |  |

## 補足

- 本書は `src/models/entities` の定義を正として作成しています。
- 制約・インデックスの最終情報はマイグレーション定義も併せて確認してください。
