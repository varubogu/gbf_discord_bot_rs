# worker.guild_last_process_times テーブル設計

## 概要

- スキーマ: `worker`
- テーブル: `guild_last_process_times`
- 実装ソース: `src/models/entities/worker/guild_last_process_times.rs`

## 主キー

- guild_id, process_type

## カラム定義（コード準拠）

| カラム | 型（Rust） | NULL許容 | 備考 |
| --- | --- | --- | --- |
| `guild_id` | `i64` | NO | 主キー |
| `process_type` | `i32` | NO | 主キー |
| `execute_time` | `Option<DateTimeUtc>` | YES |  |
| `memo` | `String` | NO |  |
| `created_at` | `DateTimeUtc` | NO |  |
| `updated_at` | `DateTimeUtc` | NO |  |

## 補足

- 本書は `src/models/entities` の定義を正として作成しています。
- 制約・インデックスの最終情報はマイグレーション定義も併せて確認してください。
