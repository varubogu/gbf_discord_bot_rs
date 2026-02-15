# worker.quest_matchings テーブル設計

## 概要

- スキーマ: `worker`
- テーブル: `quest_matchings`
- 実装ソース: `src/models/entities/worker/quest_matchings.rs`

## 主キー

- guild_id, id

## カラム定義（コード準拠）

| カラム | 型（Rust） | NULL許容 | 備考 |
| --- | --- | --- | --- |
| `guild_id` | `i64` | NO | 主キー |
| `id` | `Uuid` | NO | 主キー |
| `quest_id` | `i32` | NO |  |
| `scheduled_month` | `i32` | NO |  |
| `scheduled_day` | `i32` | NO |  |
| `scheduled_hour` | `i32` | NO |  |
| `status` | `String` | NO |  |
| `recruitment_id` | `Option<i32>` | YES |  |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## 補足

- 本書は `src/models/entities` の定義を正として作成しています。
- 制約・インデックスの最終情報はマイグレーション定義も併せて確認してください。
