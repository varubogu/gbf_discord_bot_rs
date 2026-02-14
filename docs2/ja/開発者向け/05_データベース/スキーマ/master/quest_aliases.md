# master.quest_aliases テーブル設計

## 概要

- スキーマ: `master`
- テーブル: `quest_aliases`
- 実装ソース: `src/models/entities/master/quest_aliases.rs`

## 主キー

- quest_id, sequence_no

## カラム定義（コード準拠）

| カラム | 型（Rust） | NULL許容 | 備考 |
| --- | --- | --- | --- |
| `quest_id` | `i32` | NO | 主キー |
| `sequence_no` | `i32` | NO | 主キー |
| `alias` | `String` | NO |  |
| `alias_kana_small` | `String` | NO |  |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## 補足

- 本書は `src/models/entities` の定義を正として作成しています。
- 制約・インデックスの最終情報はマイグレーション定義も併せて確認してください。
