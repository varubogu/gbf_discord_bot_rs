# guild_master.auto_recruitments テーブル設計

## 概要

- スキーマ: `guild_master`
- テーブル: `auto_recruitments`
- 実装ソース: `src/models/entities/guild_master/auto_recruitments.rs`

## 主キー

- guild_id

## カラム定義（コード準拠）

| カラム | 型（Rust） | NULL許容 | 備考 |
| --- | --- | --- | --- |
| `guild_id` | `i64` | NO | 主キー |
| `category_id` | `i64` | NO |  |
| `matching_channel_id` | `Option<i64>` | YES |  |
| `quest_channel_id` | `Option<i64>` | YES |  |
| `matching_channel_is_bot_created` | `bool` | NO |  |
| `quest_channel_is_bot_created` | `bool` | NO |  |
| `matching_message_id` | `Option<i64>` | YES |  |
| `days_range` | `i32` | NO |  |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## 補足

- 本書は `src/models/entities` の定義を正として作成しています。
- 制約・インデックスの最終情報はマイグレーション定義も併せて確認してください。
