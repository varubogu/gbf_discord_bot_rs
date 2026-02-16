# worker.scheduled_tasks テーブル設計

## 概要

- スキーマ: `worker`
- テーブル: `scheduled_tasks`
- 実装ソース: `src/models/entities/worker/scheduled_tasks.rs`（実装反映後に同期）

## 主キー

- id

## カラム定義（設計）

| カラム | 型（DB） | NULL許容 | 備考 |
| --- | --- | --- | --- |
| `id` | `serial` | NO | 主キー |
| `schedule_datetime` | `timestamptz` | NO |  |
| `task_type` | `int` | NO |  |
| `guild_id` | `bigint` | YES |  |
| `channel_id` | `bigint` | YES |  |
| `execution_status` | `worker.task_execution_status` | NO | 実行状態（デフォルト: `pending`） |
| `created_at` | `timestamptz` | NO |  |
| `updated_at` | `timestamptz` | NO |  |

## ENUM定義

### `worker.task_execution_status`

| 値 | 意味 | 次回スケジューラ実行対象 |
| --- | --- | --- |
| `pending` | 未実行 | 対象 |
| `succeeded` | 正常終了 | 対象外 |
| `succeeded_with_warning` | 正常終了（警告あり） | 対象外 |
| `failed` | 異常終了 | 対象外 |

## インデックス方針（抜粋）

- 未実行タスク取得用の部分インデックスは `execution_status = 'pending'` を条件にする

## 補足

- 本書はスケジュール実行状態の新仕様（ENUM化）を含む設計を記載しています。
- 制約・インデックスの最終情報はマイグレーション定義も併せて確認してください。
