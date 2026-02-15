# guild_master.auto_recruitment_participants テーブル設計

## 概要

- スキーマ: `guild_master`
- テーブル: `auto_recruitment_participants`
- 実装ソース: `src/models/entities/guild_master/auto_recruitment_participants.rs`

## 主キー

- guild_id, user_id, month, day, hour

## カラム定義（コード準拠）

| カラム | 型（Rust） | NULL許容 | 備考 |
| --- | --- | --- | --- |
| `guild_id` | `i64` | NO | 主キー |
| `user_id` | `i64` | NO | 主キー |
| `month` | `i32` | NO | 主キー |
| `day` | `i32` | NO | 主キー |
| `hour` | `i32` | NO | 主キー |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## 補足

- 本書は `src/models/entities` の定義を正として作成しています。
- 制約・インデックスの最終情報はマイグレーション定義も併せて確認してください。
