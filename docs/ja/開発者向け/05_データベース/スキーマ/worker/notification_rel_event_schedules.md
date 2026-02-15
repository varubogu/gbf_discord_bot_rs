# worker.notification_rel_event_schedules テーブル設計

## 概要

- スキーマ: `worker`
- テーブル: `notification_rel_event_schedules`
- 実装ソース: `src/models/entities/worker/notification_rel_event_schedules.rs`

## 主キー

- event_schedule_id, notification_id

## カラム定義（コード準拠）

| カラム | 型（Rust） | NULL許容 | 備考 |
| --- | --- | --- | --- |
| `event_schedule_id` | `Uuid` | NO | 主キー |
| `event_schedule_detail_id` | `Option<Uuid>` | YES |  |
| `notification_id` | `i32` | NO | 主キー |
| `created_at` | `DateTimeUtc` | NO |  |

## 補足

- 本書は `src/models/entities` の定義を正として作成しています。
- 制約・インデックスの最終情報はマイグレーション定義も併せて確認してください。
