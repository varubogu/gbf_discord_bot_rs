# Task Type 5: Dismissal

## Overview

`task_type = 5` is the task type for "dissolve the recruitment if participants are insufficient".
It runs linked to `battle_recruitment_dismissals` through `scheduled_task_dismissals`.

## Registration flow

`DismissalManagementService::create_recruitment_dismissals` owns registration.
Main call sites are below.

- `facades/recruitment/new_recruit.rs` (normal recruitment creation)
- `RecruitmentCreationService` (when recurring recruitment runs)

For each dismissal datetime, create:

1. `battle_recruitment_dismissals`
2. `scheduled_tasks` (`task_type=5`)
3. `scheduled_task_dismissals`

Note:

- There is a `task_type=2` comment inside the service, but the actual implementation value is `ScheduledTaskType::Dismissal` (`5`).

## Execution flow

`SchedulerManager` executes `DismissalTaskExecutor` in the `task_type=5` branch.

1. Re-check task existence and `pending`
2. Get `recruitment_dismissal_id` from `scheduled_task_dismissals`
3. Resolve target recruitment ID from `battle_recruitment_dismissals`
4. Read recruitment, participant count, and quest capacity
5. If participant count is at least capacity, do not dissolve and set `succeeded`
6. If participants are insufficient, update the recruitment message to canceled display and send a reply dissolution notice
7. Update recruitment to canceled status
8. Delete notification tasks linked to the recruitment (5 minutes before/at departure)
9. Set task to `succeeded`

## Execution status and branches

- Recruitment not found: `succeeded_with_warning`
- Recruitment already canceled: `succeeded_with_warning`
- Discord message not found: `succeeded_with_warning`
- No dissolution needed due to capacity met: `succeeded`
- Insufficient participants and dissolution completed: `succeeded`
- Execution error: handled as `failed` on `SchedulerManager`

## Implementation notes

- If canceled message text retrieval fails, it falls back to a fixed message.
- Notification deletion is performed in order: `notification_rel_battle_recruitments` -> `scheduled_tasks`; `notifications` are removed by CASCADE.

## Related tables

- `worker.scheduled_tasks`
- `worker.scheduled_task_dismissals`
- `worker.battle_recruitment_dismissals`
- `worker.battle_recruitments`
- `worker.recruitment_participants`
- `worker.notifications`
- `worker.notification_rel_battle_recruitments`

## Related documents

- [../scheduling_feature.md](../scheduling_feature.md)
- [../multi_recruitment.md](../multi_recruitment.md)
- [../scheduled_recruitment.md](../scheduled_recruitment.md)
- [../../05_database/schema/worker/scheduled_tasks.md](../../05_database/schema/worker/scheduled_tasks.md)
- [../../05_database/schema/worker/scheduled_task_dismissals.md](../../05_database/schema/worker/scheduled_task_dismissals.md)
- [../../05_database/schema/worker/battle_recruitment_dismissals.md](../../05_database/schema/worker/battle_recruitment_dismissals.md)
- [../../05_database/schema/worker/notification_rel_battle_recruitments.md](../../05_database/schema/worker/notification_rel_battle_recruitments.md)
