# guild_master.auto_recruitment_channels テーブル設計

## 概要

- スキーマ: `guild_master`
- テーブル: `auto_recruitment_channels`
- 実装ソース: `src/models/entities/guild_master/auto_recruitment_channels.rs`

## 主キー

- id

## カラム定義（コード準拠）

| カラム | 型（Rust） | NULL許容 | 備考 |
| --- | --- | --- | --- |
| `id` | `i32` | NO | 主キー |
| `guild_id` | `i64` | NO |  |
| `channel_id` | `i64` | NO |  |
| `month` | `i32` | NO |  |
| `day` | `i32` | NO |  |
| `sort_order` | `i32` | NO |  |
| `is_bot_created` | `bool` | NO |  |
| `message_id` | `Option<i64>` | YES |  |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## 補足

- 本書は `src/models/entities` の定義を正として作成しています。
- 制約・インデックスの最終情報はマイグレーション定義も併せて確認してください。
