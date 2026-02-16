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
- `schedule_datetime` is the notification execution time
- `execution_status` manages execution state (`pending` / `succeeded` / `succeeded_with_warning` / `failed`)
- Scheduler execution targets only rows where `execution_status = 'pending'`

### `worker.notifications`

- `task_id` references `scheduled_tasks.id`
- Stores `guild_id` / `channel_id` / `message_text_id`
- `is_sent` manages sent state

### `worker.notification_rel_battle_recruitments`

- Stores relations between recruitment rows and notification rows
- Used to identify regeneration targets when recruitments are changed

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
- Send failure: log it, set `scheduled_tasks.execution_status = 'failed'`, and continue
- Warning completion: if send succeeds but the result should be tracked operationally, set `scheduled_tasks.execution_status = 'succeeded_with_warning'`

## Testing notes

- Notification time calculation (boundaries: day changes, near-identical times)
- Deleting and recreating old notifications on changes
- Bulk delete on cancel
- Prevent double sends (`is_sent` / `execution_status`)

## Operational notes

- Monitor notification success rate and latency
- Monitor pending notification count (`execution_status = 'pending'`)
- Monitor failed count (`execution_status = 'failed'`) and warning-complete count (`execution_status = 'succeeded_with_warning'`)
- Verify fallback behavior when message templates are missing
