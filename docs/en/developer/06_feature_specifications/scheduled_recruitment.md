# Scheduled Recruitment

## Overview

This feature automatically creates co-op recruitments based on weekday/time schedules.
Per-guild schedules are stored in `guild_master.battle_recruitment_schedules` and executed as `task_type = RecurringRecruitment`.

## Goals

- Reduce the cost of manually creating routine recruitments
- Enable continuous recruitment posting aligned to guild operations
- Preserve retryability when scheduled execution fails

## Main commands

- `/recruitment-schedule-create`
- `/recruitment-schedule-delete`
- `/recruitment-schedule-list`

Execution is restricted to users with the `gbf_bot_control` role.

## Data model

### `guild_master.battle_recruitment_schedules`

- Core schedule body (quest, strategy, time, enabled flag)
- `recruit_start_day_offset` controls the offset day for recruitment post start
- Can be paused with `is_enabled = false`

### `guild_master.battle_recruitment_schedule_days`

- Manages execution weekdays (multiple weekdays per schedule)

### `worker.scheduled_task_recurring_recruitments`

- Stores the relationship between execution tasks and schedules

## Offset behavior

When `recruit_start_day_offset` is omitted, it is determined automatically by time comparison.

- If `recruit_start_time < quest_start_time`: `0` (same-day recruitment)
- If `recruit_start_time >= quest_start_time`: `1` (previous-day recruitment)

When manually specified, the input value is prioritized (within implementation validation bounds).

## Flow

1. Validate and persist schedule inputs
2. SchedulerManager executes `RecurringRecruitment` at due time
3. Reconstruct the current occurrence `quest_start_at` from `task.schedule_datetime`
4. If `quest_start_at <= now`, try to resolve a currently executable occurrence (`recruit_start_at <= now < quest_start_at`)
5. If found, create that occurrence immediately and complete the skipped past task as `succeeded_with_warning`
6. If not found, skip recruitment creation and only register the next task with `succeeded_with_warning`
7. If departure is still in the future, create recruitment message automatically
8. Calculate next execution datetime and register the next task
9. If `is_enabled = false`, skip execution

## Validation

- For same-day recruitment (`offset=0`), require `recruit_start_time < quest_start_time`
- Reject empty weekdays
- Return error for unknown `quest_id` / `battle_style_id`
- Reject creation when recruitment channel is not configured

## Error handling

- When recruitment creation fails, mark the execution as failed and keep next-task consistency
- Keep retryable logs for Discord send failures
- Skip orphan tasks whose schedule has already been deleted
- For delayed executions where departure already passed, optionally recover by creating the current recruitment window occurrence; complete the past task itself as `succeeded_with_warning`

## Testing notes

- Offset auto-detection logic
- Weekly datetime calculation (including year boundary)
- Skip behavior for disabled schedules
- Consistency of next-task re-registration

## Operational notes

- Monitor the number of enabled schedules
- Monitor execution failure rates
- Periodically review and remove unnecessary schedules
