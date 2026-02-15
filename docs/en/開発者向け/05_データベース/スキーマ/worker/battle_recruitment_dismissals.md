# worker.battle_recruitment_dismissals テーブル設計

## 概要

- スキーマ: `worker`
- テーブル: `battle_recruitment_dismissals`
- 実装ソース: `src/models/entities/worker/battle_recruitment_dismissals.rs`

## 主キー

- id

## カラム定義（コード準拠）

| カラム | 型（Rust） | NULL許容 | 備考 |
| --- | --- | --- | --- |
| `id` | `i32` | NO | 主キー |
| `recruitment_id` | `i32` | NO |  |
| `input_value` | `String` | NO |  |
| `input_type` | `i32` | NO |  |
| `dismissal_datetime` | `Option<DateTimeUtc>` | YES |  |
| `relative_days` | `Option<i32>` | YES |  |
| `relative_hours` | `Option<i32>` | YES |  |
| `relative_minutes` | `Option<i32>` | YES |  |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## 補足

- 本書は `src/models/entities` の定義を正として作成しています。
- 制約・インデックスの最終情報はマイグレーション定義も併せて確認してください。
