# Task Type 7: Auto Matching

## Overview

`task_type = 7` matches desired conditions of auto-recruitment participants,
and runs notifications plus recruitment creation for matched groups.
Targets are all guilds, and `scheduled_tasks.guild_id` / `channel_id` are managed as `NULL`.

## Registration flow

- Initial registration: when an auto-recruitment category is set up (`category_setup_facade`)
- Re-registration: after execution, self-register the next run 10 seconds later

At initial registration, pending tasks are checked and no duplicate is created if `task_type=7` already exists.

## Execution flow

`SchedulerManager` executes `AutoMatchingTaskExecutor` in the `task_type=7` branch.

1. Re-check task existence and `pending`
2. Execute `PeriodicMatchingService::process_matching`
   - Join available times (`auto_recruitment_participants`) with desired quests (`user_desired_quests`)
   - Group identical conditions and make groups of 2+ users as candidates
   - For six-element quests, apply grouping that avoids duplicate elements
   - Create `quest_matchings` / `quest_matching_users`
3. For each matched group, try notification send and recruitment creation
4. Set current task to `succeeded`
5. Create next task (10 seconds later)

## Notification/recruitment details

For each matched group:

1. Resolve guild settings (`auto_recruitments`) and the matching channel
2. Send notifications with participant mentions (Embed)
3. Calculate departure datetime from month/day/hour (JST base, supports `24..28` hour notation)
4. If departure datetime is in the future, create recruitment and update `quest_matchings.recruitment_id`

## Execution status and error control

- Even when zero matches are formed, set `succeeded` (`NoMatches`)
- Notification send failures are logged only, and recruitment creation continues
- Recruitment creation failures affect only that matching, and processing continues
- Fatal errors during execution are handled as `failed` on `SchedulerManager`

## Implementation notes

- If departure datetime becomes past time, recruitment creation is skipped.
- Guilds/matchings that cannot resolve quest info or auto-recruitment settings are skipped.
- Because this uses self-re-registration, monitoring should include continuity of the task chain.

## Related tables

- `worker.scheduled_tasks`
- `guild_master.auto_recruitments`
- `guild_master.auto_recruitment_participants`
- `guild_master.user_desired_quests`
- `worker.quest_matchings`
- `worker.quest_matching_users`
- `worker.battle_recruitments`

## Related documents

- [../scheduling_feature.md](../scheduling_feature.md)
- [../auto_recruitment.md](../auto_recruitment.md)
- [../../05_database/schema/worker/scheduled_tasks.md](../../05_database/schema/worker/scheduled_tasks.md)
- [../../05_database/schema/guild_master/auto_recruitments.md](../../05_database/schema/guild_master/auto_recruitments.md)
- [../../05_database/schema/guild_master/auto_recruitment_participants.md](../../05_database/schema/guild_master/auto_recruitment_participants.md)
- [../../05_database/schema/guild_master/user_desired_quests.md](../../05_database/schema/guild_master/user_desired_quests.md)
- [../../05_database/schema/worker/quest_matchings.md](../../05_database/schema/worker/quest_matchings.md)
- [../../05_database/schema/worker/quest_matching_users.md](../../05_database/schema/worker/quest_matching_users.md)
