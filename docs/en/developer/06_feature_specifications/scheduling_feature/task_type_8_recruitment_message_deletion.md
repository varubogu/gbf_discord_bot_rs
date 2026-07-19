# Task Type 8: Recruitment Message Deletion

## Overview

`task_type = 8` deletes the original Discord message for a multi recruitment after the configured delay from quest departure time.

The task does not delete `worker.battle_recruitments` rows, cancellation notification replies, dissolution replies, or any other database data. The deletion target is only the Discord recruitment post recorded by `battle_recruitments.channel_id` and `battle_recruitments.message_id`.

## Registration flow

- Manual recruitments: `/recruit` and `/recruit2` create a deletion task when the recruitment row is created.
- Recurring and automatic recruitments: flows that create rows in `worker.battle_recruitments` through the shared recruitment creation service create the same deletion task.
- Departure time changes: pending deletion tasks for the recruitment are deleted and recreated using the current setting.
- Cancellation: deletion tasks are not removed or recalculated. The original recruitment post is still deleted at the original planned deletion time.

## Delay configuration

The delay is expressed in minutes.

```text
MULTI_RECRUITMENT_DELETE_AFTER_DEPARTURE_MINUTES
```

Resolution order:

1. Guild environment variable in `guild_master.guild_environments`
2. Global environment variable in `master.environments`
3. Program default `10080` minutes (7 days)

Unset values fall through to the next source. Empty strings, non-numeric values, and values less than or equal to 0 also fall through and are logged as warnings. No user-facing message is emitted for configuration fallbacks.

Changing the setting affects newly created recruitments and recruitments whose departure time is changed after the setting update. Existing pending deletion tasks are not automatically recalculated.

## Execution flow

`SchedulerTaskDispatchFacade` executes `RecruitmentMessageDeletionTaskExecutor` in the `task_type=8` branch.

1. Re-check task existence and `pending`
2. Load `worker.scheduled_task_recruitment_message_deletions` by `task_id`
3. Load the target recruitment through `BattleRecruitmentsRepository::get_by_id_with_txn`
4. If `message_id != 0`, call `DiscordMessageGateway::delete_message(channel_id, message_id)`
5. Mark the task by result

## Execution status and error control

- Successful Discord deletion: `succeeded`
- Missing relation row: `succeeded_with_warning`
- Missing recruitment row: `succeeded_with_warning`
- `message_id = 0`: `succeeded_with_warning`
- Discord NotFound (`10008` message missing, or equivalent gateway `NotFound`): `succeeded_with_warning`
- Permission errors and transient Discord API failures: `failed`

## Related tables

- `worker.scheduled_tasks`
- `worker.scheduled_task_recruitment_message_deletions`
- `worker.battle_recruitments`
- `guild_master.guild_environments`
- `master.environments`

## Related documents

- [../scheduling_feature.md](../scheduling_feature.md)
- [../multi_recruitment.md](../multi_recruitment.md)
- [../../05_database/schema/worker/scheduled_tasks.md](../../05_database/schema/worker/scheduled_tasks.md)
