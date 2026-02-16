# Task Type 6: Auto Recruitment Date Rotation

## Overview

`task_type = 6` updates date channels for auto recruitment on a daily basis.
Targets are all guilds, and `scheduled_tasks.guild_id` / `channel_id` are managed as `NULL`.

## Registration flow

- Initial registration: when an auto-recruitment category is set up (`category_setup_facade`)
- Re-registration: after execution, self-register the next run at 00:00 JST on the next day

At initial registration, pending tasks are checked and no duplicate is created if `task_type=6` already exists.

## Execution flow

`SchedulerManager` executes `AutoRecruitmentRotationTaskExecutor` in the `task_type=6` branch.

1. Re-check task existence and `pending`
2. Read all `auto_recruitment_channels`
3. Process channels by date for each guild
4. Update past-date channels to `current max date + 1 day`
5. After DB date updates, rename Discord channels to `M-D`
6. Reorder channel positions
   (`matching=0` -> date channels `1..n` -> `quest=n+1`)
7. Set current task to `succeeded`
8. Create next task (00:00 JST next day)

## Execution status and error control

- Even when there are zero rotation targets, set `succeeded` and create the next task
- Channel rename failures are logged only, then processing continues
- Channel reorder failures are also logged only, then processing continues
- Fatal errors during execution are handled as `failed` on `SchedulerManager`

## Implementation notes

- Date judgment is based on JST (`UTC + 9`).
- Invalid dates (for example, 2/30) are skipped.
- It includes comparison logic that handles year boundaries.

## Related tables

- `worker.scheduled_tasks`
- `guild_master.auto_recruitment_channels`
- `guild_master.auto_recruitments`

## Related documents

- [../scheduling_feature.md](../scheduling_feature.md)
- [../auto_recruitment.md](../auto_recruitment.md)
- [../../05_database/schema/worker/scheduled_tasks.md](../../05_database/schema/worker/scheduled_tasks.md)
- [../../05_database/schema/guild_master/auto_recruitment_channels.md](../../05_database/schema/guild_master/auto_recruitment_channels.md)
- [../../05_database/schema/guild_master/auto_recruitments.md](../../05_database/schema/guild_master/auto_recruitments.md)
