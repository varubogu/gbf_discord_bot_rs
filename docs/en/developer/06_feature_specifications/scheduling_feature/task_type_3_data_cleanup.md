# Task Type 3: Data Cleanup

## Overview

`task_type = 3` is the task type for data cleanup.
It is executed from the scheduler dispatch path and reuses `DataCleanupService`.

## Scheduler path

`TaskDispatchService` handles `task_type=3` as follows.

1. Re-check `scheduled_task_cleanups` by `task_id`
2. If no relation exists, continue with a warning log
3. Execute `DataCleanupService`
4. Mark the task as `succeeded` on success, or `failed` on fatal error

Transaction boundaries are managed by `SchedulerTaskDispatchFacade`.

## Cleanup execution path

Cleanup logic is shared with the maintenance batch (`src/bin/cleanup.rs`) via `DataCleanupService`.

### Execution conditions

- Retention period: `CLEANUP_RETENTION_DAYS` (defaults to 30 days when not set)
- Executors:
  - Scheduler runtime (`task_type=3`)
  - Maintenance batch (`src/bin/cleanup.rs`)

### Deletion scope (`DataCleanupService`)

1. Delete expired/completed data from `battle_recruitments`
2. Delete expired/non-pending data from `scheduled_tasks`
   (except `task_type=3`)
3. `notifications` are cleaned via CASCADE delete from `scheduled_tasks`
   (`cleanup_notifications` currently returns `0`)

## Implementation notes

- `scheduled_task_cleanups` is currently used for consistency checking/logging in the scheduler path.
- Actual deletion policy is controlled by `DataCleanupService` and `CLEANUP_RETENTION_DAYS`.

## Related tables

- `worker.scheduled_tasks`
- `worker.scheduled_task_cleanups`
- `worker.battle_recruitments`
- `worker.notifications`

## Related documents

- [../scheduling_feature.md](../scheduling_feature.md)
- [../../05_database/schema/worker/scheduled_tasks.md](../../05_database/schema/worker/scheduled_tasks.md)
- [../../05_database/schema/worker/scheduled_task_cleanups.md](../../05_database/schema/worker/scheduled_task_cleanups.md)
- [../../05_database/schema/worker/battle_recruitments.md](../../05_database/schema/worker/battle_recruitments.md)
- [../../05_database/schema/worker/notifications.md](../../05_database/schema/worker/notifications.md)
