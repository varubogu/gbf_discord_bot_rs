# worker.quest_matching_users テーブル設計

## 概要

- スキーマ: `worker`
- テーブル: `quest_matching_users`
- 実装ソース: `src/models/entities/worker/quest_matching_users.rs`

## 主キー

- guild_id, matching_id, user_id

## カラム定義（コード準拠）

| カラム | 型（Rust） | NULL許容 | 備考 |
| --- | --- | --- | --- |
| `guild_id` | `i64` | NO | 主キー |
| `matching_id` | `Uuid` | NO | 主キー |
| `user_id` | `i64` | NO | 主キー |
| `battle_style_id` | `Option<i32>` | YES |  |
| `joined_at` | `DateTimeUtc` | NO |  |
| `left_at` | `Option<DateTimeUtc>` | YES |  |

## 補足

- 本書は `src/models/entities` の定義を正として作成しています。
- 制約・インデックスの最終情報はマイグレーション定義も併せて確認してください。
