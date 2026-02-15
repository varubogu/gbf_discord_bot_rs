# guild_master.battle_recruitment_schedule_days テーブル設計

## 概要

- スキーマ: `guild_master`
- テーブル: `battle_recruitment_schedule_days`
- 実装ソース: `src/models/entities/guild_master/battle_recruitment_schedule_days.rs`

## 主キー

- id

## カラム定義（コード準拠）

| カラム | 型（Rust） | NULL許容 | 備考 |
| --- | --- | --- | --- |
| `id` | `i32` | NO | 主キー |
| `schedule_id` | `i32` | NO |  |
| `day_of_week` | `i32` | NO |  |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## 補足

- 本書は `src/models/entities` の定義を正として作成しています。
- 制約・インデックスの最終情報はマイグレーション定義も併せて確認してください。
