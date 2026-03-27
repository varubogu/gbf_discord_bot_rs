# guild_master.auto_recruitment_match_rules テーブル設計

## 概要

- スキーマ: `guild_master`
- テーブル: `auto_recruitment_match_rules`
- 実装ソース: `src/models/entities/guild_master/auto_recruitment_match_rules.rs`

## 主キー

- guild_id, quest_id

## カラム定義（コード準拠）

| カラム | 型（Rust） | NULL許容 | 備考 |
| --- | --- | --- | --- |
| `guild_id` | `i64` | NO | 主キー |
| `quest_id` | `i32` | NO | 主キー |
| `preset_type` | `String` | NO | マッチングプリセット名 |
| `min_match_count` | `i32` | NO | 成立に必要な最低人数 |
| `required_battle_style_id` | `Option<i32>` | YES | `specific_element_n_plus_any` でのみ使用 |
| `required_battle_style_count` | `Option<i32>` | YES | `specific_element_n_plus_any` でのみ使用 |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## 補足

- 本書は `src/models/entities` の定義を正として作成しています。
- 制約・インデックスの最終情報はマイグレーション定義も併せて確認してください。
