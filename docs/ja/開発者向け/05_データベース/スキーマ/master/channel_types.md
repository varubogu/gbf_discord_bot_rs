# master.channel_types テーブル設計

## 概要

- スキーマ: `master`
- テーブル: `channel_types`
- 実装ソース: `src/models/entities/master/channel_types.rs`

## 主キー

- id

## カラム定義（コード準拠）

| カラム | 型（Rust） | NULL許容 | 備考 |
| --- | --- | --- | --- |
| `id` | `i32` | NO | 主キー |
| `name` | `String` | NO |  |
| `memo` | `Option<String>` | YES |  |

## 補足

- 本書は `src/models/entities` の定義を正として作成しています。
- 制約・インデックスの最終情報はマイグレーション定義も併せて確認してください。
