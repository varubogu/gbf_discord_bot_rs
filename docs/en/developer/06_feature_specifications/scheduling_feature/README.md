# Scheduling Feature (By Task Type)

## Overview

This directory provides task-type-specific design documentation organized by `worker.scheduled_tasks.task_type`.
For common specifications (execution cycle, execution statuses, and overall architecture), see [../scheduling_feature.md](../scheduling_feature.md).

## Task type list

- `1: Notification`
  - [task_type_1_notification.md](task_type_1_notification.md)
  - [event_schedule_notifications.md](event_schedule_notifications.md)
- `2: Dissolution`
  - [task_type_2_dissolution.md](task_type_2_dissolution.md)
- `3: DataCleanup`
  - [task_type_3_data_cleanup.md](task_type_3_data_cleanup.md)
- `4: RecurringRecruitment`
  - [task_type_4_recurring_recruitment.md](task_type_4_recurring_recruitment.md)
- `5: Dismissal`
  - [task_type_5_dismissal.md](task_type_5_dismissal.md)
- `6: AutoRecruitmentRotation`
  - [task_type_6_auto_recruitment_rotation.md](task_type_6_auto_recruitment_rotation.md)
- `7: AutoMatching`
  - [task_type_7_auto_matching.md](task_type_7_auto_matching.md)
- `8: RecruitmentMessageDeletion`
  - [task_type_8_recruitment_message_deletion.md](task_type_8_recruitment_message_deletion.md)

## Notes

- `task_type=1` is used by both event notifications and recruitment notifications.
- `task_type=3` is executed from the scheduler path via `SchedulerTaskDispatchFacade` and shares cleanup logic with `src/bin/cleanup.rs`.

## Implementation boundaries

- Scheduling repository traits are defined in `src/repository/schedule/**`.
- SeaORM implementations are placed in `src/infrastructure/database/repositories/schedule/**`.
- Concrete repository types are wired only in `src/di/repositories.rs`.
- Facades own transaction boundaries (begin/commit/rollback) and pass transactions to services.
