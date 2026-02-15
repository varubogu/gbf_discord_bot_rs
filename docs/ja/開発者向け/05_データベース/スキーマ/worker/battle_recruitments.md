# worker.battle_recruitments テーブル設計

## 概要

- スキーマ: `worker`
- テーブル: `battle_recruitments`
- 実装ソース: `src/models/entities/worker/battle_recruitments.rs`

## 主キー

- id

## カラム定義（コード準拠）

| カラム | 型（Rust） | NULL許容 | 備考 |
| --- | --- | --- | --- |
| `id` | `i32` | NO | 主キー |
| `guild_id` | `i64` | NO |  |
| `channel_id` | `i64` | NO |  |
| `message_id` | `i64` | NO |  |
| `quest_id` | `i32` | NO |  |
| `battle_style_id` | `i32` | NO |  |
| `quest_start_at` | `DateTimeUtc` | NO |  |
| `is_recruiting` | `bool` | NO |  |
| `is_canceled` | `bool` | NO |  |
| `recruit_end_message_id` | `Option<i64>` | YES |  |
| `full_notification_sent` | `bool` | NO |  |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## 補足

- 本書は `src/models/entities` の定義を正として作成しています。
- 制約・インデックスの最終情報はマイグレーション定義も併せて確認してください。
