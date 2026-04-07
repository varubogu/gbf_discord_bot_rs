# Task Type 7: Auto Matching

## Overview

`task_type = 7` matches desired conditions of auto-recruitment participants,
and runs notifications plus recruitment creation for matched groups.
Targets are all guilds, and `scheduled_tasks.guild_id` / `channel_id` are managed as `NULL`.

## Registration flow

- Initial registration: when an auto-recruitment category is set up (`category_setup_facade`)
- Re-registration: after execution, self-register the next run 10 seconds later

At initial registration, pending tasks are checked and no duplicate is created if `task_type=7` already exists.

## Scheduler integration dependency direction

- `TaskDispatchService` is the composition point. It composes `PeriodicMatchingService` and `AutoMatchingTaskExecutor` using repositories provided by `crate::di::Repositories`.
- `SchedulerTaskDispatchFacade` owns transaction boundaries and invokes `TaskDispatchService`.
- Concrete repository implementations are SeaORM adapters under `src/infrastructure/database/repositories/**`.
- `PeriodicMatchingService` and `AutoMatchingTaskExecutor` depend on repository traits via `crate::repository` (auto-recruitment, schedule, recruitment, and master data ports), not on `SeaOrm*Repository` concrete types.
- Keep the one-way flow: `scheduler_manager (trigger) -> facade (tx) -> task_dispatch_service (composition) -> service/executor -> repository ports`.
- Keep concrete adapter placement unified under `src/infrastructure/database/repositories/**`.

## Implementation reference paths

```text
src/services/schedule/scheduler_manager.rs
src/services/schedule/task_dispatch_service.rs
src/facades/schedule/scheduler_task_dispatch_facade.rs
src/services/schedule/auto_matching_task_executor.rs
src/services/auto_recruitment/matching_service.rs
src/repository/auto_recruitment/
src/repository/schedule/scheduled_task_repository.rs
src/repository/battle_recruitments_repository.rs
src/repository/quest_repository.rs
src/infrastructure/database/repositories/auto_recruitment/
src/infrastructure/database/repositories/schedule/scheduled_task_repository.rs
src/infrastructure/database/repositories/recruitment/battle_recruitments_repository.rs
src/infrastructure/database/repositories/master_data/quest_repository.rs
src/di/repositories.rs
```

## Execution flow

`TaskDispatchService` executes `AutoMatchingTaskExecutor` in the `task_type=7` branch.

1. Re-check task existence and `pending`
2. Execute `PeriodicMatchingService::process_matching`
   - Join available times (`auto_recruitment_participants`) with desired quests (`user_desired_quests`)
   - Resolve presets from `auto_recruitment_match_rules` with priority `guild_id -> global(guild_id=0)`
   - Compile preset rules into required slots and build the smallest successful groups
   - If no rule exists for the quest, keep the legacy `2+ users` behavior
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
5. If recruitment creation succeeds, edit the previously sent notification and append a jump link to the recruitment post

- The recruitment post UI uses the same shared generation path as `/recruit_new_v2`

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
- `guild_master.auto_recruitment_match_rules`
- `guild_master.auto_recruitment_match_rule_quotas`
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
- [../../05_database/schema/guild_master/auto_recruitment_match_rules.md](../../05_database/schema/guild_master/auto_recruitment_match_rules.md)
- [../../05_database/schema/guild_master/auto_recruitment_match_rule_quotas.md](../../05_database/schema/guild_master/auto_recruitment_match_rule_quotas.md)
- [../../05_database/schema/guild_master/user_desired_quests.md](../../05_database/schema/guild_master/user_desired_quests.md)
- [../../05_database/schema/worker/quest_matchings.md](../../05_database/schema/worker/quest_matchings.md)
- [../../05_database/schema/worker/quest_matching_users.md](../../05_database/schema/worker/quest_matching_users.md)
