# master.quests テーブル設計

## 概要

- スキーマ: `master`
- テーブル: `quests`
- 実装ソース: `src/models/entities/master/quests.rs`

## 主キー

- id

## カラム定義（コード準拠）

| カラム | 型（Rust） | NULL許容 | 備考 |
| --- | --- | --- | --- |
| `id` | `i32` | NO | 主キー |
| `name` | `String` | NO |  |
| `default_battle_style_id` | `i32` | NO |  |
| `recruit_count` | `i32` | NO |  |
| `available_battle_style_ids` | `String` | NO |  |
| `sort_order` | `i32` | NO |  |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## 補足

- 本書は `src/models/entities` の定義を正として作成しています。
- 制約・インデックスの最終情報はマイグレーション定義も併せて確認してください。
