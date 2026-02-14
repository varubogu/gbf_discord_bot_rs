# master.message_texts テーブル設計

## 概要

- スキーマ: `master`
- テーブル: `message_texts`
- 実装ソース: `src/models/entities/master/message_texts.rs`

## 主キー

- id

## カラム定義（コード準拠）

| カラム | 型（Rust） | NULL許容 | 備考 |
| --- | --- | --- | --- |
| `id` | `String` | NO | 主キー |
| `message_jp` | `String` | NO |  |
| `message_en` | `Option<String>` | YES |  |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## 補足

- 本書は `src/models/entities` の定義を正として作成しています。
- 制約・インデックスの最終情報はマイグレーション定義も併せて確認してください。
