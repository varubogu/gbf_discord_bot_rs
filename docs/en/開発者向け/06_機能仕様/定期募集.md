# Scheduled Recruitment

## Overview

This feature automatically creates co-op recruitments based on weekday/time schedules.
Per-guild schedules are stored in `guild_master.battle_recruitment_schedules` and executed as `task_type = RecurringRecruitment`.

## Goals

- Reduce the cost of manually creating routine recruitments
- Enable continuous recruitment posting aligned to guild operations
- Preserve retryability when scheduled execution fails

## Main commands

- `/定期募集作成`
- `/定期募集削除`
- `/定期募集一覧`

実行権限は `gbf_bot_control` ロール保持者を前提とします。

## Data model

### `guild_master.battle_recruitment_schedules`

- スケジュール本体（クエスト、戦術、時刻、有効フラグ）
- `recruit_start_day_offset` で募集開始日のオフセットを管理
- `is_enabled = false` で一時停止可能

### `guild_master.battle_recruitment_schedule_days`

- 実行曜日を管理（1スケジュールに複数曜日）

### `worker.scheduled_task_recurring_recruitments`

- 実行タスクとスケジュールの関連を保持

## Offset behavior

`recruit_start_day_offset` の省略時は、時刻比較で自動決定します。

- If `recruit_start_time < quest_start_time`: `0` (same-day recruitment)
- If `recruit_start_time >= quest_start_time`: `1` (previous-day recruitment)

手動指定時は入力値を優先します（実装上のバリデーション範囲に従う）。

## Flow

1. スケジュール登録時に入力を検証し保存
2. SchedulerManager が対象時刻で `RecurringRecruitment` タスクを実行
3. 募集メッセージを自動作成
4. 次回実行日時を計算し、次のタスクを再登録
5. `is_enabled = false` なら実行せずスキップ

## Validation

- 当日募集（offset=0）の場合、`recruit_start_time < quest_start_time` を要求
- 曜日未指定は禁止
- 存在しない `quest_id` / `battle_style_id` はエラー
- 募集チャンネル未設定時は作成を拒否

## Error handling

- 募集作成失敗時は当該実行を失敗として記録し、次回タスクとの整合性を維持
- Discord送信失敗時は再試行可能なログを残す
- スケジュール削除済みの孤立タスクは実行時再確認でスキップ

## Testing notes

- オフセット自動判定ロジック
- 週次実行日時の計算（年跨ぎ含む）
- 無効化スケジュールのスキップ
- 次回タスク再登録の整合性

## Operational notes

- 有効スケジュール件数の監視
- 実行失敗率の監視
- 定期的な不要スケジュール棚卸し
