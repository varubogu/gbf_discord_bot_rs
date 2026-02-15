# Schema and Table Design Overview

## Purpose

This page summarizes the schema layout in the current implementation and provides links to each table’s design notes.

## Design policy

- The primary source of truth is the code in `src/models/entities`.
- `docs/develop/database` and `docs/develop/design/database` are referenced as supplementary information.
- If older design documents conflict, prioritize the code definitions.

## Schema list (code-aligned)

| Schema | Role | # Tables | Index |
| --- | --- | ---: | --- |
| `master` | Shared reference/master data | 9 | [master/README.md](./schema/master/README.md) |
| `guild_master` | Per-guild configuration and operational data | 20 | [guild_master/README.md](./schema/guild_master/README.md) |
| `worker` | Runtime work data, schedules, and recruitment history | 15 | [worker/README.md](./schema/worker/README.md) |

- Total tables: 44

## Table designs by schema

### master
- `battle_styles`: [battle_styles.md](./schema/master/battle_styles.md)
- `channel_types`: [channel_types.md](./schema/master/channel_types.md)
- `elements`: [elements.md](./schema/master/elements.md)
- `environments`: [environments.md](./schema/master/environments.md)
- `event_schedule_details`: [event_schedule_details.md](./schema/master/event_schedule_details.md)
- `event_schedules`: [event_schedules.md](./schema/master/event_schedules.md)
- `message_texts`: [message_texts.md](./schema/master/message_texts.md)
- `quest_aliases`: [quest_aliases.md](./schema/master/quest_aliases.md)
- `quests`: [quests.md](./schema/master/quests.md)

### guild_master
- `all_recruitment_notification_roles`: [all_recruitment_notification_roles.md](./schema/guild_master/all_recruitment_notification_roles.md)
- `auto_recruitment_channels`: [auto_recruitment_channels.md](./schema/guild_master/auto_recruitment_channels.md)
- `auto_recruitment_participants`: [auto_recruitment_participants.md](./schema/guild_master/auto_recruitment_participants.md)
- `auto_recruitment_quest_messages`: [auto_recruitment_quest_messages.md](./schema/guild_master/auto_recruitment_quest_messages.md)
- `auto_recruitments`: [auto_recruitments.md](./schema/guild_master/auto_recruitments.md)
- `battle_recruitment_schedule_days`: [battle_recruitment_schedule_days.md](./schema/guild_master/battle_recruitment_schedule_days.md)
- `battle_recruitment_schedule_dismissals`: [battle_recruitment_schedule_dismissals.md](./schema/guild_master/battle_recruitment_schedule_dismissals.md)
- `battle_recruitment_schedules`: [battle_recruitment_schedules.md](./schema/guild_master/battle_recruitment_schedules.md)
- `guild_channels`: [guild_channels.md](./schema/guild_master/guild_channels.md)
- `guild_environments`: [guild_environments.md](./schema/guild_master/guild_environments.md)
- `guild_event_schedule_details`: [guild_event_schedule_details.md](./schema/guild_master/guild_event_schedule_details.md)
- `guild_event_schedules`: [guild_event_schedules.md](./schema/guild_master/guild_event_schedules.md)
- `guild_message_texts`: [guild_message_texts.md](./schema/guild_master/guild_message_texts.md)
- `guild_quest_disables`: [guild_quest_disables.md](./schema/guild_master/guild_quest_disables.md)
- `guild_settings`: [guild_settings.md](./schema/guild_master/guild_settings.md)
- `guild_spreadsheet_exports`: [guild_spreadsheet_exports.md](./schema/guild_master/guild_spreadsheet_exports.md)
- `guild_spreadsheet_imports`: [guild_spreadsheet_imports.md](./schema/guild_master/guild_spreadsheet_imports.md)
- `guilds`: [guilds.md](./schema/guild_master/guilds.md)
- `quest_recruitment_notification_roles`: [quest_recruitment_notification_roles.md](./schema/guild_master/quest_recruitment_notification_roles.md)
- `user_desired_quests`: [user_desired_quests.md](./schema/guild_master/user_desired_quests.md)

### worker
- `battle_recruitment_dismissals`: [battle_recruitment_dismissals.md](./schema/worker/battle_recruitment_dismissals.md)
- `battle_recruitments`: [battle_recruitments.md](./schema/worker/battle_recruitments.md)
- `guild_last_process_times`: [guild_last_process_times.md](./schema/worker/guild_last_process_times.md)
- `last_process_times`: [last_process_times.md](./schema/worker/last_process_times.md)
- `notification_rel_battle_recruitments`: [notification_rel_battle_recruitments.md](./schema/worker/notification_rel_battle_recruitments.md)
- `notification_rel_event_schedules`: [notification_rel_event_schedules.md](./schema/worker/notification_rel_event_schedules.md)
- `notifications`: [notifications.md](./schema/worker/notifications.md)
- `quest_matching_users`: [quest_matching_users.md](./schema/worker/quest_matching_users.md)
- `quest_matchings`: [quest_matchings.md](./schema/worker/quest_matchings.md)
- `recruitment_participants`: [recruitment_participants.md](./schema/worker/recruitment_participants.md)
- `scheduled_task_cleanups`: [scheduled_task_cleanups.md](./schema/worker/scheduled_task_cleanups.md)
- `scheduled_task_dismissals`: [scheduled_task_dismissals.md](./schema/worker/scheduled_task_dismissals.md)
- `scheduled_task_dissolutions`: [scheduled_task_dissolutions.md](./schema/worker/scheduled_task_dissolutions.md)
- `scheduled_task_recurring_recruitments`: [scheduled_task_recurring_recruitments.md](./schema/worker/scheduled_task_recurring_recruitments.md)
- `scheduled_tasks`: [scheduled_tasks.md](./schema/worker/scheduled_tasks.md)
