# Scheduling Platform

## Overview

This feature is a shared platform for “executing processing at a specified time”, centered around `worker.scheduled_tasks`.
It uses a common execution model not only for notifications, but also for dissolutions, scheduled recruitments, and auto-recruitment tasks.

## Goals

- Standardize time-based execution on `scheduled_tasks` to simplify implementation and operations
- Separate per-feature execution logic into executors for extensibility
- Keep an execution model where failures or missing data do not cascade to other tasks

## Task types

Major variants of `ScheduledTaskType`:

- `1: Notification` send notifications
- `2: Dissolution` dissolve recruitments
- `3: DataCleanup` data cleanup
- `4: RecurringRecruitment` execute scheduled recruitments
- `5: Dismissal` dissolve due to insufficient participants
- `6: AutoRecruitmentRotation` rotate auto-recruitment dates
- `7: AutoMatching` run auto-matching

## Task execution status (`scheduled_tasks.execution_status`)

Task execution status is managed by PostgreSQL ENUM (`worker.task_execution_status`).

- `pending`: not executed yet
- `succeeded`: completed successfully
- `succeeded_with_warning`: completed successfully with warning(s)
- `failed`: completed with error

Policy:

- Scheduler execution targets only `pending`
- `succeeded_with_warning` is treated as completed and is not automatically retried
- `failed` is treated as an error completion and is not automatically retried (recover separately when needed)

## Event Notification Schedule Generation Specification (global/guild merge)

This section defines merge rules between `global` and `guild` data when regenerating event notifications (`task_type = Notification`).

### Tables used for schedule generation

The following six tables are referenced when generating event notifications.

1. `master.event_schedules` (global event schedules)
2. `master.event_schedule_details` (global in-period event detail schedules)
3. `master.message_texts` (global message texts)
4. `guild_master.guild_event_schedules` (guild event schedules)
5. `guild_master.guild_event_schedule_details` (guild in-period event detail schedules)
6. `guild_master.guild_message_texts` (guild message texts)

### Global/guild identity rules

- `event_schedules` family: rows with the same `id` are treated as the same schedule
- `event_schedule_details` family: rows with the same `id` are treated as the same detail
- `message_texts` family: rows with the same `id` are treated as the same message
- However, when joining schedules and details, `profile` equivalence is used, and schedule/detail `id` values are not join conditions

### Merge policy

- Global only: generate notifications from global data
- Guild only: generate notifications from guild data only
- Both global and guild exist: guild data overrides global within the same `id`

### Notification channel resolution priority

Resolve the destination channel in this order.

1. `guild_event_schedule_details.notification_channel_id`
2. Resolve `guild_event_schedule_details.notification_channel_type` via `guild_channels`
3. Resolve `event_schedule_details.notification_channel_type` via `guild_channels`
4. If 1-3 cannot resolve, treat as an error

Assumptions:

- This specification assumes `guild_event_schedule_details` has nullable `notification_channel_id`
- Before schema rollout, only `notification_channel_type`-based resolution is available

Notes:

- If both `notification_channel_id` and `notification_channel_type` are set in guild detail, prioritize `notification_channel_id`
- If both are missing in guild detail, do not register that detail notification for the guild (log a warning and continue)

### Message text ID resolution priority

Resolve the `message_text_id` bound to each notification row in this order.

1. Use guild detail `message_text_id` if defined
2. Otherwise, use global detail `message_text_id` if defined
3. If neither 1 nor 2 can be resolved, treat as an error

After deciding `message_text_id`, text body resolution (via `MessageService`: `guild_message_texts` -> `message_texts` -> `locales/messages.yml` -> error) and locale selection follow [Message resolution](./message_resolution.md).

### Notification regeneration flow

1. Initialize existing notification schedules (`worker.scheduled_tasks` / `worker.notifications` / `worker.notification_rel_event_schedules`)
2. Merge global and guild `event_schedules`
3. Merge global and guild `event_schedule_details`
4. For each guild, resolve destination channel and `message_text_id` from merged rows
5. Create `scheduled_tasks` / `notifications` / `notification_rel_event_schedules` only for resolved rows
6. Record error rows and continue; output aggregated results after completion

## Event Notification Error Control

### Basic policy

- Record enough context to identify which data and which error occurred
- Continue schedule generation for rows other than the failed row

### Error unit

- `event_schedules` (global/guild) errors: per event schedule row
- `event_schedule_details` (global/guild) errors: per event detail row
- Message unresolved errors: per event detail row
- Notification channel unresolved errors: per event detail row (including guild context)

### Data to keep on error

- `guild_id` (if applicable)
- `event_schedule_id`
- `event_schedule_detail_id`
- `message_text_id` (if already known during resolution)
- Error type (for example, `NotificationChannelNotResolved`, `MessageTextNotResolved`)

## Pattern List (Normal)

Numbers (1-6) used below refer to [Tables used for schedule generation](#tables-used-for-schedule-generation).
Pattern descriptions are listed in order: Event -> Detail -> Message.

- 1,2,3 exist; 4,5,6 do not exist
  - Global-only pattern
  - Send message 3 to destinations from 1,2 date/time and channels

- 1,2,3,6 exist; 4,5 do not exist
  - Event data is global, only message is guild-customized
  - Send message 6 to destinations from 1,2 date/time and channels

- 1,2,3,4,5,6 exist
  - Guild fully overrides global
  - Send message 6 to destinations from 4,5 date/time and channels

- 1,2,3,4,5 exist; 6 does not exist
  - Guild overrides global, but message text remains global
  - Send message 3 to destinations from 4,5 date/time and channels

- 1,5,3 exist; 4,2,6 do not exist
  - Event base date is global; in-period detail is guild-defined; message is global
  - Based on event date from 1, send message 3 to date/time and channels from 5

- 1,5,6 exist; 2,4,3 do not exist
  - Event base date is global; in-period detail and message are guild-defined
  - Based on event date from 1, send message 6 to date/time and channels from 5

- 4,5,6 exist; 1,2,3 do not exist
  - Guild-only event notification pattern
  - Send message 6 to destinations from 4,5 date/time and channels

## Pattern List (Errors)

- For `message_text_id` resolved from 2 or 5, the body is undefined in all of 6,3, and `locales/messages.yml`
  - Pattern where message content cannot be resolved
  - See [Message resolution](./message_resolution.md)
  - Record as message unresolved error at event detail row level

- In 2 or 5 (without explicit channel ID), channel type is specified, but no matching `guild_channels` row exists in that guild
  - Pattern where destination channel cannot be resolved
  - See [Notification channel resolution priority](#notification-channel-resolution-priority)
  - Register notifications only for resolvable guilds; record errors for unresolved guilds and continue

## Execution architecture

### Layer responsibilities

- Events: scheduler trigger, logging, exception boundary
- Facade: transaction boundary, coordination of executor calls
- Services: SchedulerManager / business logic in each TaskExecutor
- Repository: persistence for `scheduled_tasks` and related tables

### Execution cycle (every 10 seconds)

1. Fetch tasks with `execution_status = 'pending'` (past range + current to 20 seconds ahead)
2. Extract tasks with `schedule_datetime <= now` as execution targets
3. Switch executors by `task_type` and process
4. Update `scheduled_tasks.execution_status` by execution result (`succeeded` / `succeeded_with_warning` / `failed`)
5. For recurring features (for example, scheduled recruitment), generate next task

### Consistency policy

- Re-check the DB right before execution and safely skip disabled/deleted targets
- On individual task failure, log an error, update that task to `failed`, and continue processing other tasks
- Record warning-completed tasks as `succeeded_with_warning` so operations can trace them
- Prevent duplicate execution of the same task by managing state with `execution_status`

## Key tables

| Table | Role |
| --- | --- |
| `worker.scheduled_tasks` | Base table that manages execution time/type/state |
| `worker.notifications` | Notification contents (1:1 with `scheduled_tasks` via `task_id`) |
| `worker.scheduled_task_dissolutions` | Relation to dissolution targets |
| `worker.scheduled_task_dismissals` | Relation to insufficient-participant dismissal targets |
| `worker.scheduled_task_recurring_recruitments` | Relation to scheduled recruitment schedules |
| `worker.scheduled_task_cleanups` | Cleanup target information |

## Typical use cases

- Regenerate event notifications and send on schedule
- Auto-create recruitments from scheduled recruitment schedules
- Determine dissolution at recruitment departure time
- Periodic execution for auto-recruitment (rotation/matching)

## Operational notes

- Run large-scale regeneration during low-traffic hours
- Monitor the count of unexecuted tasks to detect backlog
- Misconfiguration of `DataCleanup` has high impact; manage target tables explicitly
- When regenerating notifications, always record error and skip counts so operators can trace them

## Related documents

- [Scheduling platform](scheduling_feature.md) (this document)
- [Scheduled recruitment](scheduled_recruitment.md)
- [Recruitment notifications](recruitment_notifications.md)
- [Spreadsheet integration](spreadsheet_integration.md)
- [Message resolution](message_resolution.md)
- [event_schedules.md](../05_database/schema/master/event_schedules.md)
- [event_schedule_details.md](../05_database/schema/master/event_schedule_details.md)
- [guild_event_schedules.md](../05_database/schema/guild_master/guild_event_schedules.md)
- [guild_event_schedule_details.md](../05_database/schema/guild_master/guild_event_schedule_details.md)
- [message_texts.md](../05_database/schema/master/message_texts.md)
- [guild_message_texts.md](../05_database/schema/guild_master/guild_message_texts.md)
- [scheduled_tasks.md](../05_database/schema/worker/scheduled_tasks.md)
