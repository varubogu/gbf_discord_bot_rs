# Recruitment Notifications

## Overview

This feature schedules and sends notifications for co-op recruitments (e.g., pre-departure reminders).
Notification data is stored in `worker.notifications` and `worker.notification_rel_battle_recruitments`, and execution time is stored in `worker.scheduled_tasks`.

## Goals

- Reduce missed departures by reminding participants
- Integrate notifications into the scheduling platform for easier operations and extension
- Guarantee notification consistency when recruitments are changed/canceled

## Basic specification

- Default reminder: 5 minutes before departure
- Message template: `RECRUIT_DEPARTURE_REMINDER`
- Recipients: recruitment participants and related mention targets
- Executor: SchedulerManager (every 10 seconds)

## Data model

### `worker.scheduled_tasks`

- `task_type = Notification`
- `schedule_datetime` が通知実行時刻
- `is_executed` で実行状態を管理

### `worker.notifications`

- `task_id` で `scheduled_tasks.id` を参照
- `guild_id` / `channel_id` / `message_text_id` を保持
- `is_sent` で送信済み状態を管理

### `worker.notification_rel_battle_recruitments`

- 募集レコードと通知レコードの関連を保持
- 募集変更時の再生成対象特定に使用

## Flow

### On recruitment creation

1. Save the recruitment
2. Compute notification time from departure time (-5 minutes)
3. Create a `scheduled_tasks` row (Notification)
4. Create a `notifications` row
5. Create a `notification_rel_battle_recruitments` row

### On recruitment change

1. Identify notifications via existing relations
2. Delete existing notification (relation + notification + task)
3. Regenerate notifications using the new departure time

### On recruitment cancel

1. Delete related notifications
2. Do not leave pending tasks behind

## Error handling

- Notification creation failure: rollback in the same transaction as recruitment creation
- Notification update failure: rollback the whole update
- Send failure: log it and keep a retryable state according to retry policy

## Testing notes

- Notification time calculation (boundaries: day changes, near-identical times)
- Deleting and recreating old notifications on changes
- Bulk delete on cancel
- Prevent double sends (`is_sent` / `is_executed`)

## Operational notes

- Monitor notification success rate and latency
- Monitor the count of unexecuted notifications
- Verify fallback behavior when message templates are missing
