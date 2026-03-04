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

1. Validate and persist schedule inputs
2. SchedulerManager executes `RecurringRecruitment` at due time
3. Reconstruct the current occurrence `quest_start_at` from `task.schedule_datetime`
4. If `quest_start_at <= now`, try to resolve a currently executable occurrence (`recruit_start_at <= now < quest_start_at`)
5. If found, create that occurrence immediately and complete the skipped past task as `succeeded_with_warning`
6. If not found, skip recruitment creation and only register the next task with `succeeded_with_warning`
7. If departure is still in the future, create recruitment message automatically
8. Calculate next execution datetime and register the next task
9. If `is_enabled = false`, skip execution

## Validation

- For same-day recruitment (`offset=0`), require `recruit_start_time < quest_start_time`
- Reject empty weekdays
- Return error for unknown `quest_id` / `battle_style_id`
- Reject creation when recruitment channel is not configured

## Error handling

- When recruitment creation fails, mark the execution as failed and keep next-task consistency
- Keep retryable logs for Discord send failures
- Skip orphan tasks whose schedule has already been deleted
- For delayed executions where departure already passed, optionally recover by creating the current recruitment window occurrence; complete the past task itself as `succeeded_with_warning`

## Testing notes

- Offset auto-detection logic
- Weekly datetime calculation (including year boundary)
- Skip behavior for disabled schedules
- Consistency of next-task re-registration

## Operational notes

- 有効スケジュール件数の監視
- 実行失敗率の監視
- 定期的な不要スケジュール棚卸し
