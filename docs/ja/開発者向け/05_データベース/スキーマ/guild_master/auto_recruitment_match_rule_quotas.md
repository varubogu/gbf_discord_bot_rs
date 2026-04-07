# guild_master.auto_recruitment_match_rule_quotas テーブル設計

## 概要

- スキーマ: `guild_master`
- テーブル: `auto_recruitment_match_rule_quotas`
- 実装ソース: `src/models/entities/guild_master/auto_recruitment_match_rule_quotas.rs`

## 主キー

- guild_id, quest_id, battle_style_id

## カラム定義（コード準拠）

| カラム | 型（Rust） | NULL許容 | 備考 |
| --- | --- | --- | --- |
| `guild_id` | `i64` | NO | 主キー（`0` はグローバル設定） |
| `quest_id` | `i32` | NO | 主キー |
| `battle_style_id` | `i32` | NO | 主キー |
| `required_count` | `i32` | NO | 当該属性に必要な人数 |
| `sort_order` | `i32` | NO | 表示・処理順の安定化に使用 |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## 補足

- 本書は `src/models/entities` の定義を正として作成しています。
- 制約・インデックスの最終情報はマイグレーション定義も併せて確認してください。
- `auto_recruitment_match_rules` と同じスコープ（同一 `guild_id`）の明細を参照します。
