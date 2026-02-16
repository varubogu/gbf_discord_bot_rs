# Event Schedule Notifications

## Overview

This document defines specifications specialized for `task_type = Notification` (event schedule notifications).
For scheduler common infrastructure (execution status, execution cycle, consistency policy, and others), see [../scheduling_feature.md](../scheduling_feature.md).

## Global/guild merge specification

This section defines merge rules between `global` and `guild` data when regenerating event notifications.

### Tables used for schedule generation

The following six tables are referenced when generating event notifications.

1. `master.event_schedules` (global event schedules)
2. `master.event_schedule_details` (global in-period event detail schedules)
3. `master.message_texts` (global message texts)
4. `guild_master.guild_event_schedules` (guild event schedules)
5. `guild_master.guild_event_schedule_details` (guild in-period event detail schedules)
6. `guild_master.guild_message_texts` (guild message texts)

### Global/guild identity rules

- `event_schedules` family: rows with the same `id` are treated as the same schedule
- `event_schedule_details` family: rows with the same `id` are treated as the same detail
- `message_texts` family: rows with the same `id` are treated as the same message
- However, when joining schedules and details, `profile` equivalence is used, and schedule/detail `id` values are not join conditions

### Merge policy

- Global only exists: generate notifications from global data
- Guild only exists: generate notifications from guild data only
- Both global and guild exist: guild data overrides global within the same `id`

## Resolution rules

### Notification channel resolution priority

Resolve destination channels in this order.

1. `guild_event_schedule_details.notification_channel_id`
2. Resolve `guild_event_schedule_details.notification_channel_type` via `guild_channels`
3. Resolve `event_schedule_details.notification_channel_type` via `guild_channels`
4. If none of 1 to 3 can resolve, treat as an error

Assumptions:

- This specification assumes `guild_event_schedule_details` has nullable `notification_channel_id`
- Before schema rollout, only `notification_channel_type`-based resolution is available

Notes:

- If both `notification_channel_id` and `notification_channel_type` are set in guild detail, prioritize `notification_channel_id`
- If both are missing in guild detail, do not register that detail notification for the guild (log a warning and continue)

### Message text ID resolution priority

Resolve `message_text_id` bound to each notification row in this order.

1. Use guild detail `message_text_id` if defined
2. Use global detail `message_text_id` if defined
3. If neither 1 nor 2 can be resolved, treat as an error

After deciding `message_text_id`, message body resolution (`MessageService`: `guild_message_texts` -> `message_texts` -> `locales/messages.yml` -> error) and locale selection follow [../message_resolution.md](../message_resolution.md).

## Notification regeneration flow

1. Initialize existing notification schedules (`worker.scheduled_tasks` / `worker.notifications` / `worker.notification_rel_event_schedules`)
2. Merge global and guild `event_schedules`
3. Merge global and guild `event_schedule_details`
4. For each guild, resolve destination channel and `message_text_id` from merged rows
5. Create `scheduled_tasks` / `notifications` / `notification_rel_event_schedules` only for resolved rows
6. Record error rows and continue; output aggregated results after completion

Note (linkage to `notification_rel_event_schedules`):

- Link `event_schedule_id` only when it exists in `master.event_schedules.id`
- Link `event_schedule_detail_id` only when it exists in `master.event_schedule_details.id`
- For guild-only IDs (IDs that exist only in `guild_event_schedules` / `guild_event_schedule_details`),
  create `scheduled_tasks` / `notifications` but leave `notification_rel_event_schedules` columns unset (NULL) or create no row

## Event notification error control

### Basic policy

- Record enough context to identify which data and which error occurred
- Continue schedule generation for rows other than the failed row

### Error unit

- Caused by `event_schedules` (global/guild): per event schedule row
- Caused by `event_schedule_details` (global/guild): per event detail row
- Caused by unresolved message text: per event detail row
- Caused by unresolved destination channel: per event detail row (including guild context)

### Data to keep on error

- `guild_id` (if applicable)
- `event_schedule_id`
- `event_schedule_detail_id`
- `message_text_id` (if already known during resolution)
- Error type (for example, `NotificationChannelNotResolved`, `MessageTextNotResolved`)

## Pattern list (normal)

The numbers (1 to 6) below refer to [Tables used for schedule generation](#tables-used-for-schedule-generation).
Pattern descriptions are listed in this order: Event -> Detail -> Message.

- 1,2,3 exist; 4,5,6 do not exist
  - Global-only pattern
  - Send 3 to date/time and channels from 1,2

- 1,2,3,6 exist; 4,5 do not exist
  - Event data is global and only message is guild-customized
  - Send 6 to date/time and channels from 1,2

- 1,2,3,4,5,6 exist
  - Guild fully overrides global
  - Send 6 to date/time and channels from 4,5

- 1,2,3,4,5 exist; 6 does not exist
  - Guild overrides global but message text remains global
  - Send 3 to date/time and channels from 4,5

- 1,5,3 exist; 4,2,6 do not exist
  - Event base date is global, in-period detail is guild-defined, and message is global
  - Based on event date from 1, send 3 to date/time and channels from 5

- 1,5,6 exist; 2,4,3 do not exist
  - Event base date is global, and both in-period detail and message are guild-defined
  - Based on event date from 1, send 6 to date/time and channels from 5

- 4,5,6 exist; 1,2,3 do not exist
  - Guild-only event notification pattern
  - Send 6 to date/time and channels from 4,5

## Pattern list (errors)

- For the `message_text_id` chosen in 2 or 5, the body is undefined in all of 6, 3, and `locales/messages.yml`
  - Pattern where message content cannot be resolved
  - See [../message_resolution.md](../message_resolution.md)
  - Must be recorded as an unresolved message error per detail row

- In 2 or 5 (without explicit destination channel ID), channel type is specified, but no matching `guild_channels` row exists in that guild
  - Pattern where destination channel cannot be resolved
  - See [Notification channel resolution priority](#notification-channel-resolution-priority)
  - Register notifications only for resolvable guilds, and continue after recording errors for unresolved guilds

## Related documents

- [../scheduling_feature.md](../scheduling_feature.md)
- [../recruitment_notifications.md](../recruitment_notifications.md)
- [../message_resolution.md](../message_resolution.md)
- [../../05_database/schema/master/event_schedules.md](../../05_database/schema/master/event_schedules.md)
- [../../05_database/schema/master/event_schedule_details.md](../../05_database/schema/master/event_schedule_details.md)
- [../../05_database/schema/guild_master/guild_event_schedules.md](../../05_database/schema/guild_master/guild_event_schedules.md)
- [../../05_database/schema/guild_master/guild_event_schedule_details.md](../../05_database/schema/guild_master/guild_event_schedule_details.md)
- [../../05_database/schema/master/message_texts.md](../../05_database/schema/master/message_texts.md)
- [../../05_database/schema/guild_master/guild_message_texts.md](../../05_database/schema/guild_master/guild_message_texts.md)
- [../../05_database/schema/worker/scheduled_tasks.md](../../05_database/schema/worker/scheduled_tasks.md)
