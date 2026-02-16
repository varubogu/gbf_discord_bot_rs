# Task Type 1: Notification

## Overview

`task_type = 1` is the notification sending task type.
In the current implementation, it is mainly used in the following two flows.

- Event schedule notifications (notify based on merged global/guild data)
- Co-op recruitment departure notifications (5 minutes before and at departure)

## Registration flow

### 1) Event schedule notifications

`SchedulerService::generate_and_persist_schedules` creates:

1. `scheduled_tasks` (`task_type=1`)
2. `notifications`
3. `notification_rel_event_schedules`

For details, see [event_schedule_notifications.md](event_schedule_notifications.md).

### 2) Co-op recruitment departure notifications

`NotificationManagementService::create_recruitment_departure_notification` creates:

1. `scheduled_tasks` (`task_type=1`)
2. `notifications`
3. `notification_rel_battle_recruitments`

Notes:

- The 5-minute-prior notification is created only when its datetime is in the future
- The at-departure notification is always created

## Execution flow

`SchedulerManager` executes the following in the `task_type=1` branch.

1. Read `notifications` by `task_id`
2. Execute `NotificationService::send_single_notification`
3. On successful send, set `scheduled_tasks.execution_status = succeeded`
4. On failed send, set `failed`

Key points in `send_single_notification`:

- `message_text_id == "-"` is treated as a skipped send and success
- Notifications with `notification_rel_battle_recruitments` are sent as recruitment notifications (reply style)
- Notifications without that relation are sent as normal notifications
- After sending, update `notifications.is_sent = true`

## Execution status and error control

- `notifications` not found: `succeeded_with_warning` (warn about inconsistency and do not auto-retry)
- Send processing error: `failed`
- Notification query error: `failed`

## Related tables

- `worker.scheduled_tasks`
- `worker.notifications`
- `worker.notification_rel_event_schedules`
- `worker.notification_rel_battle_recruitments`

## Related documents

- [../scheduling_feature.md](../scheduling_feature.md)
- [event_schedule_notifications.md](event_schedule_notifications.md)
- [../recruitment_notifications.md](../recruitment_notifications.md)
- [../../05_database/schema/worker/scheduled_tasks.md](../../05_database/schema/worker/scheduled_tasks.md)
- [../../05_database/schema/worker/notifications.md](../../05_database/schema/worker/notifications.md)
- [../../05_database/schema/worker/notification_rel_event_schedules.md](../../05_database/schema/worker/notification_rel_event_schedules.md)
- [../../05_database/schema/worker/notification_rel_battle_recruitments.md](../../05_database/schema/worker/notification_rel_battle_recruitments.md)
