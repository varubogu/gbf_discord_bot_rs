# Task Type 3: Data Cleanup

## Overview

`task_type = 3` is the task type for data cleanup.
The `scheduled_task_cleanups` table and repository exist, but the execution path in `SchedulerManager` is not implemented.

## Scheduler path (current)

The `task_type=3` branch in `SchedulerManager` currently behaves as follows.

1. Output warning log: `DataCleanup task is not implemented`
2. Update the target task to `failed`

Therefore, `task_type=3` in `scheduled_tasks` is currently not executable as-is.

## Current cleanup execution path

Actual cleanup is executed by starting `DataCleanupService` from `src/bin/cleanup.rs`.

### Execution conditions

- Retention period: `CLEANUP_RETENTION_DAYS` (defaults to 30 days when not set)
- Executor: maintenance batch (assumed maintenance container)

### Deletion scope (`DataCleanupService`)

1. Delete expired/completed data from `battle_recruitments`
2. Delete expired/non-pending data from `scheduled_tasks`
   (except `task_type=3`)
3. `notifications` are cleaned via CASCADE delete from `scheduled_tasks`
   (`cleanup_notifications` currently returns `0`)

## Implementation notes

- `scheduled_task_cleanups` is not referenced or updated by the current flow.
- If this is integrated into the scheduler in the future, add a dedicated executor to the `task_type=3` branch.

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
