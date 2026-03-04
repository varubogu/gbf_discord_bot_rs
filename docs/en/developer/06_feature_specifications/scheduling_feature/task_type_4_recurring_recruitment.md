# Task Type 4: Recurring Recruitment

## Overview

`task_type = 4` executes recurring recruitment schedules and auto-creates recruitments.
It is linked to target schedules through `scheduled_task_recurring_recruitments`.

## Registration and deletion flow

### Registration

- On schedule creation (`ScheduleCreateService`)
- On schedule enable (`ScheduleCommandService::enable_schedule`)
- On post-execution next reservation (`RecurringRecruitmentTaskExecutor::create_next_scheduled_task`)

At registration, update these two tables together.

1. `scheduled_tasks` (`task_type=4`)
2. `scheduled_task_recurring_recruitments` (`scheduled_task_id` <-> `recruitment_schedule_id`)

### Deletion

- Delete only pending tasks when a schedule is disabled or deleted
- Also delete related `scheduled_task_recurring_recruitments`

## Execution flow

`SchedulerManager` executes `RecurringRecruitmentTaskExecutor` in the `task_type=4` branch.

1. Re-check task existence and `pending`
2. Get target schedule ID from `scheduled_task_recurring_recruitments`
3. Read schedule body (including weekdays)
4. If schedule is missing/disabled, finish with `succeeded_with_warning`
5. Reconstruct `CalculatedRecruitmentTime` for the current execution from `task.schedule_datetime`
6. If `quest_start_at <= now`, check whether there is a currently executable occurrence (`recruit_start_at <= now < quest_start_at`)
7. If such occurrence exists, create that recruitment immediately and mark the skipped task as `succeeded_with_warning`
8. If no executable occurrence exists, skip recruitment creation, register only the next execution task, and mark the current task as `succeeded_with_warning`
9. Only when departure is still in the future, create recruitment by `RecruitmentCreationService::create_recruitment_from_schedule`
10. Calculate/register the next execution task (search up to 365 days ahead)
11. Set current task to `succeeded` when recruitment was created without skipping a past task

## Implementation notes

- Recruitment time is reconstructed from `CalculatedRecruitmentTime` resolved by `task.schedule_datetime`; `Utc::now()` is no longer directly assigned.
- When execution is delayed (for example bot downtime) and `quest_start_at <= now`, recruitment creation is skipped.
- If execution is delayed but currently falls in the next occurrence recruitment window, that occurrence is created immediately.
- Next execution datetime is searched in 7-day steps, and only the first future datetime found is registered.

## Execution status and error control

- Schedule already deleted: `succeeded_with_warning`
- Schedule already disabled: `succeeded_with_warning`
- Completed normally: `succeeded`
- Error during execution: handled as `failed` on `SchedulerManager`

## Related tables

- `worker.scheduled_tasks`
- `worker.scheduled_task_recurring_recruitments`
- `guild_master.battle_recruitment_schedules`
- `guild_master.battle_recruitment_schedule_days`
- `worker.battle_recruitments`

## Related documents

- [../scheduling_feature.md](../scheduling_feature.md)
- [../scheduled_recruitment.md](../scheduled_recruitment.md)
- [../../05_database/schema/worker/scheduled_tasks.md](../../05_database/schema/worker/scheduled_tasks.md)
- [../../05_database/schema/worker/scheduled_task_recurring_recruitments.md](../../05_database/schema/worker/scheduled_task_recurring_recruitments.md)
- [../../05_database/schema/guild_master/battle_recruitment_schedules.md](../../05_database/schema/guild_master/battle_recruitment_schedules.md)
- [../../05_database/schema/guild_master/battle_recruitment_schedule_days.md](../../05_database/schema/guild_master/battle_recruitment_schedule_days.md)
