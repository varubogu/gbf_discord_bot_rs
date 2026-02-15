# guild_master.guild_message_texts テーブル設計

## 概要

- スキーマ: `guild_master`
- テーブル: `guild_message_texts`
- 実装ソース: `src/models/entities/guild_master/guild_message_texts.rs`

## 主キー

- guild_id, id

## カラム定義（コード準拠）

| カラム | 型（Rust） | NULL許容 | 備考 |
| --- | --- | --- | --- |
| `guild_id` | `i64` | NO | 主キー |
| `id` | `String` | NO | 主キー |
| `message_jp` | `String` | NO |  |
| `message_en` | `Option<String>` | YES |  |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## 補足

- 本書は `src/models/entities` の定義を正として作成しています。
- 制約・インデックスの最終情報はマイグレーション定義も併せて確認してください。
