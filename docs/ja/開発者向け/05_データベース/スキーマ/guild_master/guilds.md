# guild_master.guilds テーブル設計

## 概要

- スキーマ: `guild_master`
- テーブル: `guilds`
- 実装ソース: `src/models/entities/guild_master/guilds.rs`

## 主キー

- guild_id

## カラム定義（コード準拠）

| カラム | 型（Rust） | NULL許容 | 備考 |
| --- | --- | --- | --- |
| `guild_id` | `i64` | NO | 主キー |
| `name` | `String` | NO |  |
| `recruit_channel_id` | `Option<i64>` | YES |  |
| `timezone` | `Option<String>` | YES |  |
| `default_recruit_duration` | `Option<i32>` | YES |  |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## 補足

- 本書は `src/models/entities` の定義を正として作成しています。
- 制約・インデックスの最終情報はマイグレーション定義も併せて確認してください。
