# Task Type 2: Dissolution

## Overview

`task_type = 2` is the recruitment dissolution task type linked by `scheduled_task_dissolutions`.
`DissolutionTaskExecutor` updates the recruitment message and sends dissolution notifications.

## Registration flow (current)

Implementation-wise, `ScheduledTaskDissolutionRepository::create` exists,
but there is currently no confirmed service/facade-layer call that creates `ScheduledTaskType::Dissolution`.

As a result, current execution requires at least the following two records in advance.

1. `scheduled_tasks` (`task_type=2`)
2. `scheduled_task_dissolutions` (`task_id` <-> `recruit_id`)

## Execution flow

`SchedulerManager` executes `DissolutionTaskExecutor` in the `task_type=2` branch.

1. Re-check task existence and `execution_status = pending`
2. Get target recruitment ID from `scheduled_task_dissolutions`
3. Read recruitment data
4. Read the recruitment message and update it to canceled display
5. Update the recruitment to canceled status
6. Send a reply dissolution notification with participant mentions
7. Update task to `succeeded`

## Execution status and branches

- Recruitment not found: `succeeded_with_warning`
- Recruitment already canceled: `succeeded_with_warning`
- Discord message not found: `succeeded_with_warning`
- Dissolution processing completed normally: `succeeded`
- Task/relation inconsistency or execution error: handled as `failed` on `SchedulerManager`

## Implementation notes

- On cancellation, `battle_recruitments.cancel_message_id` is updated with `0`.
- `DissolutionExecutionResult::SkippedDueToSufficientParticipants` is defined, but there is no current return path that uses it.

## Related tables

- `worker.scheduled_tasks`
- `worker.scheduled_task_dissolutions`
- `worker.battle_recruitments`
- `worker.recruitment_participants`

## Related documents

- [../scheduling_feature.md](../scheduling_feature.md)
- [../multi_recruitment.md](../multi_recruitment.md)
- [../../05_database/schema/worker/scheduled_tasks.md](../../05_database/schema/worker/scheduled_tasks.md)
- [../../05_database/schema/worker/scheduled_task_dissolutions.md](../../05_database/schema/worker/scheduled_task_dissolutions.md)
- [../../05_database/schema/worker/battle_recruitments.md](../../05_database/schema/worker/battle_recruitments.md)
- [../../05_database/schema/worker/recruitment_participants.md](../../05_database/schema/worker/recruitment_participants.md)
