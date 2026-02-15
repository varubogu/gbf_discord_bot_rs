# guild_master.user_desired_quests テーブル設計

## 概要

- スキーマ: `guild_master`
- テーブル: `user_desired_quests`
- 実装ソース: `src/models/entities/guild_master/user_desired_quests.rs`

## 主キー

- guild_id, user_id, quest_id, battle_style_id

## カラム定義（コード準拠）

| カラム | 型（Rust） | NULL許容 | 備考 |
| --- | --- | --- | --- |
| `guild_id` | `i64` | NO | 主キー |
| `user_id` | `i64` | NO | 主キー |
| `quest_id` | `i32` | NO | 主キー |
| `battle_style_id` | `i32` | NO | 主キー |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## 補足

- 本書は `src/models/entities` の定義を正として作成しています。
- 制約・インデックスの最終情報はマイグレーション定義も併せて確認してください。
