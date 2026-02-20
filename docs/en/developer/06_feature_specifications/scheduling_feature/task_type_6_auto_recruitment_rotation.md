# Task Type 6: Auto Recruitment Date Rotation

## Overview

`task_type = 6` updates date channels for auto recruitment on a daily basis.
Targets are all guilds, and `scheduled_tasks.guild_id` / `channel_id` are managed as `NULL`.

## Registration flow

- Initial registration: when an auto-recruitment category is set up (`category_setup_facade`)
- Re-registration: after execution, self-register the next run at 00:00 JST on the next day

At initial registration, pending tasks are checked and no duplicate is created if `task_type=6` already exists.

## Scheduler integration dependency direction

- `SchedulerManager` is the composition point. It receives concrete repositories from `crate::di::Repositories` (SeaORM adapters in `src/infrastructure/database/repositories/**`).
- `AutoRecruitmentRotationTaskExecutor` depends on repository traits via `crate::repository` (`ScheduledTaskRepository`, `AutoRecruitmentChannelRepository`, `AutoRecruitmentRepository`), not on concrete SeaORM types.
- Keep the one-way flow: `scheduler_manager (composition) -> executor -> repository ports`.
- Keep concrete adapter placement unified under `src/infrastructure/database/repositories/**`.

## Implementation reference paths

```text
src/services/schedule/scheduler_manager.rs
src/services/schedule/auto_recruitment_rotation_task_executor.rs
src/repository/auto_recruitment/auto_recruitment_channel_repository.rs
src/repository/auto_recruitment/auto_recruitment_repository.rs
src/repository/schedule/scheduled_task_repository.rs
src/infrastructure/database/repositories/auto_recruitment/auto_recruitment_channel_repository.rs
src/infrastructure/database/repositories/auto_recruitment/auto_recruitment_repository.rs
src/infrastructure/database/repositories/schedule/scheduled_task_repository.rs
src/di/repositories.rs
```

## Execution flow

`SchedulerManager` executes `AutoRecruitmentRotationTaskExecutor` in the `task_type=6` branch.

1. Re-check task existence and `pending`
2. Read all `auto_recruitment_channels`
3. Sort channels by logical date for each guild
4. Reassign unique contiguous dates `today + 0..n-1` to each channel
5. After DB date updates, rename Discord channels to `M-D`
6. Reorder channel positions
   (`matching=0` -> date channels `1..n` -> `quest=n+1`)
7. Set current task to `succeeded`
8. Create next task (00:00 JST next day)

## Startup repair flow

`SchedulerManager::start` runs `repair_on_startup` once during startup.

1. Load all rows from `auto_recruitments`
2. For each guild, if date channels are fewer than `days_range`, create missing channels
3. Do not delete excess channels; reassign contiguous future dates across all existing channels
4. Reorder category channel positions
5. Log failures and continue bot startup

## Execution status and error control

- Even when there are zero rotation targets, set `succeeded` and create the next task
- Channel rename failures are logged only, then processing continues
- Channel reorder failures are also logged only, then processing continues
- Fatal errors during execution are handled as `failed` on `SchedulerManager`

## Implementation notes

- Date judgment is based on JST (`UTC + 9`).
- Invalid dates (for example, 2/30) are skipped.
- It includes comparison logic that handles year boundaries.
- This reassignment avoids collapsing multiple channels into the same date after long downtime.

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
