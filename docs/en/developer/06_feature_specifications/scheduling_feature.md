# Scheduling Platform

## Overview

This is a shared platform for “executing processing at a specified time”, centered around `worker.scheduled_tasks`.
It uses the same execution model not only for notifications, but also for dissolutions, scheduled recruitments, and auto-recruitment tasks.

## Goals

- Standardize time-based execution on `scheduled_tasks` to simplify implementation and operations
- Separate per-feature execution logic into executors for extensibility
- Keep an execution model where failures/missing data do not cascade to other tasks

## Task types

Major variants of `ScheduledTaskType`:

- `1: Notification` send notifications
- `2: Dissolution` dissolve recruitments
- `3: DataCleanup` data cleanup
- `4: RecurringRecruitment` execute scheduled recruitments
- `5: Dismissal` dissolve due to insufficient participants
- `6: AutoRecruitmentRotation` rotate auto-recruitment dates
- `7: AutoMatching` run auto-matching

## Execution architecture

### Layer responsibilities

- Events: scheduler trigger, logging, exception boundary
- Facade: transaction boundary, coordination of executor calls
- Services: SchedulerManager / business logic in each TaskExecutor
- Repository: persistence for `scheduled_tasks` and related tables

### Execution cycle (every 10 seconds)

1. `is_executed = false` のタスクを取得（過去分 + 現在〜20秒先）
2. `schedule_datetime <= now` のタスクを実行対象として抽出
3. `task_type` ごとに Executor を切り替えて処理
4. 成功時は `scheduled_tasks.is_executed = true` を更新
5. 定期募集など再実行が必要なものは次回タスクを生成

### Consistency policy

- Re-check the DB right before execution and safely skip disabled/deleted targets
- On individual task failure, log an error and continue processing other tasks
- Prevent duplicate execution of the same task by managing state with executed flags

## Key tables

| Table | Role |
| --- | --- |
| `worker.scheduled_tasks` | Base table that manages execution time/type/state |
| `worker.notifications` | Notification contents (1:1 with `scheduled_tasks` via `task_id`) |
| `worker.scheduled_task_dissolutions` | Relation to dissolutions |
| `worker.scheduled_task_dismissals` | Relation to insufficient-participant dismissals |
| `worker.scheduled_task_recurring_recruitments` | Relation to scheduled recruitment schedules |
| `worker.scheduled_task_cleanups` | Cleanup target info |

## Typical use cases

- Regenerate event notifications and send on schedule
- Auto-create recruitments from scheduled recruitment schedules
- Decide dissolution at recruitment departure time
- Periodic execution for auto-recruitment (rotation/matching)

## Operational notes

- Run large-scale regeneration during low-traffic hours
- Monitor the count of unexecuted tasks to detect backlog
- Misconfiguration of `DataCleanup` has high impact; manage target tables explicitly

## Related documents

- [Scheduling platform](scheduling_feature.md) (this document)
- [scheduled_recruitment.md](scheduled_recruitment.md)
- [recruitment_notifications.md](recruitment_notifications.md)
- [scheduled_tasks.md](../05_database/schema/worker/scheduled_tasks.md)
