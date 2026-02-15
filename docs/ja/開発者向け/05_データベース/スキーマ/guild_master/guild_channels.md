# guild_master.guild_channels テーブル設計

## 概要

- スキーマ: `guild_master`
- テーブル: `guild_channels`
- 実装ソース: `src/models/entities/guild_master/guild_channels.rs`

## 主キー

- guild_id, channel_type

## カラム定義（コード準拠）

| カラム | 型（Rust） | NULL許容 | 備考 |
| --- | --- | --- | --- |
| `guild_id` | `i64` | NO | 主キー |
| `channel_type` | `i32` | NO | 主キー |
| `channel_id` | `i64` | NO |  |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## 補足

- 本書は `src/models/entities` の定義を正として作成しています。
- 制約・インデックスの最終情報はマイグレーション定義も併せて確認してください。
