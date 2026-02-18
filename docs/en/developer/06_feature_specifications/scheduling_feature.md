# Scheduling Feature

## Overview

This feature is a shared platform for "executing processing at a specified time", centered around `worker.scheduled_tasks`.
It uses a common execution model not only for notifications, but also for dissolutions, recurring recruitments, and auto-recruitment tasks.

## Goals

- Standardize time-based execution on `scheduled_tasks` to simplify implementation and operations
- Separate per-feature execution logic into executors for extensibility
- Keep an execution model where failures or missing data do not cascade to other tasks

## Task types

Major variants of `ScheduledTaskType`:

- `1: Notification` send notifications
- `2: Dissolution` dissolve recruitments
- `3: DataCleanup` data cleanup
- `4: RecurringRecruitment` execute recurring recruitments
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

## Detailed specification by task type

- `1: Notification`: [task_type_1_notification.md](scheduling_feature/task_type_1_notification.md)
- `2: Dissolution`: [task_type_2_dissolution.md](scheduling_feature/task_type_2_dissolution.md)
- `3: DataCleanup`: [task_type_3_data_cleanup.md](scheduling_feature/task_type_3_data_cleanup.md)
- `4: RecurringRecruitment`: [task_type_4_recurring_recruitment.md](scheduling_feature/task_type_4_recurring_recruitment.md)
- `5: Dismissal`: [task_type_5_dismissal.md](scheduling_feature/task_type_5_dismissal.md)
- `6: AutoRecruitmentRotation`: [task_type_6_auto_recruitment_rotation.md](scheduling_feature/task_type_6_auto_recruitment_rotation.md)
- `7: AutoMatching`: [task_type_7_auto_matching.md](scheduling_feature/task_type_7_auto_matching.md)

## Execution architecture

### Layer responsibilities

- Events: scheduler trigger, logging, exception boundary
- Facade: transaction boundary, coordination of executor calls
- Services: SchedulerManager / business logic in each TaskExecutor
- Repository: persistence for `scheduled_tasks` and related tables

### Repository/implementation mapping

- Trait (port) definitions for scheduling: `src/repository/schedule/**`
- SeaORM repository implementations: `src/infrastructure/database/repositories/schedule/**`
- DI wiring point for concrete adapters: `src/di/repositories.rs`

Rules:

- `services` and `facades` depend on `crate::repository::schedule::*` traits only.
- `Facade` starts, commits, and rolls back transactions; services receive `DatabaseTransaction` from facades.
- Non-DI layers must not instantiate `SeaOrm*Repository` directly.

## Execution cycle (every 10 seconds)

1. Fetch tasks with `execution_status = 'pending'` (past range + current to 20 seconds ahead)
2. Extract tasks with `schedule_datetime <= now` as execution targets
3. Switch executors by `task_type` and process
4. Update `scheduled_tasks.execution_status` by execution result (`succeeded` / `succeeded_with_warning` / `failed`)
5. For recurring features, generate the next task when needed

## Consistency policy

- Re-check the DB right before execution and safely skip disabled/deleted targets
- On individual task failure, log an error, update that task to `failed`, and continue processing other tasks
- Record warning-completed tasks as `succeeded_with_warning` so operations can trace them
- Prevent duplicate execution of the same task by managing state with `execution_status`

## Key tables

| Table | Role |
| --- | --- |
| `worker.scheduled_tasks` | Base table that manages execution time, type, and status |
| `worker.notifications` | Notification contents (1:1 with `scheduled_tasks` via `task_id`) |
| `worker.scheduled_task_dissolutions` | Relation to dissolution target recruitments |
| `worker.scheduled_task_dismissals` | Relation to insufficient-participant dismissal targets |
| `worker.scheduled_task_recurring_recruitments` | Relation to recurring recruitment schedules |
| `worker.scheduled_task_cleanups` | Cleanup target information |

## Typical use cases

- Regenerate event notifications and send them on time
- Auto-create recruitments from recurring recruitment schedules
- Decide dissolution at recruitment departure time
- Periodic execution for auto-recruitment (rotation/matching)

## Operational notes

- Run large-scale regeneration during low-traffic hours
- Monitor the count of unexecuted tasks to detect backlogs
- Misconfiguration of `DataCleanup` has high impact, so manage target tables explicitly
- When regenerating notifications, always record error and skip counts so operators can trace them

## Related documents

- [scheduling_feature.md](scheduling_feature.md) (this document)
- [scheduling_feature/README.md](scheduling_feature/README.md)
- [scheduling_feature/event_schedule_notifications.md](scheduling_feature/event_schedule_notifications.md)
- [scheduling_feature/task_type_1_notification.md](scheduling_feature/task_type_1_notification.md)
- [scheduling_feature/task_type_2_dissolution.md](scheduling_feature/task_type_2_dissolution.md)
- [scheduling_feature/task_type_3_data_cleanup.md](scheduling_feature/task_type_3_data_cleanup.md)
- [scheduling_feature/task_type_4_recurring_recruitment.md](scheduling_feature/task_type_4_recurring_recruitment.md)
- [scheduling_feature/task_type_5_dismissal.md](scheduling_feature/task_type_5_dismissal.md)
- [scheduling_feature/task_type_6_auto_recruitment_rotation.md](scheduling_feature/task_type_6_auto_recruitment_rotation.md)
- [scheduling_feature/task_type_7_auto_matching.md](scheduling_feature/task_type_7_auto_matching.md)
- [scheduled_recruitment.md](scheduled_recruitment.md)
- [recruitment_notifications.md](recruitment_notifications.md)
- [spreadsheet_integration.md](spreadsheet_integration.md)
- [message_resolution.md](message_resolution.md)
- [event_schedules.md](../05_database/schema/master/event_schedules.md)
- [event_schedule_details.md](../05_database/schema/master/event_schedule_details.md)
- [guild_event_schedules.md](../05_database/schema/guild_master/guild_event_schedules.md)
- [guild_event_schedule_details.md](../05_database/schema/guild_master/guild_event_schedule_details.md)
- [message_texts.md](../05_database/schema/master/message_texts.md)
- [guild_message_texts.md](../05_database/schema/guild_master/guild_message_texts.md)
- [scheduled_tasks.md](../05_database/schema/worker/scheduled_tasks.md)
