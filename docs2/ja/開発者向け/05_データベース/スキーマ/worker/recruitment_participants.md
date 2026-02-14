# worker.recruitment_participants テーブル設計

## 概要

- スキーマ: `worker`
- テーブル: `recruitment_participants`
- 実装ソース: `src/models/entities/worker/recruitment_participants.rs`

## 主キー

- id

## カラム定義（コード準拠）

| カラム | 型（Rust） | NULL許容 | 備考 |
| --- | --- | --- | --- |
| `id` | `i64` | NO | 主キー |
| `recruitment_id` | `i32` | NO |  |
| `user_id` | `i64` | NO |  |
| `element_id` | `Option<i32>` | YES |  |
| `participated_at` | `DateTimeUtc` | NO |  |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## 補足

- 本書は `src/models/entities` の定義を正として作成しています。
- 制約・インデックスの最終情報はマイグレーション定義も併せて確認してください。
