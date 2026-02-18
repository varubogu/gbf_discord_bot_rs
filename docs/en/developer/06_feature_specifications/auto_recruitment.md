# Auto-Matching for Co-op Recruitment (Design)

## Overview



Users pre-register desired quests and available time slots. When multiple users match on the same date/time and quest, the bot automatically forms a match and creates a recruitment post.
This feature uses a Discord category: “date channels” manage time availability per day, and a “quest channel” manages desired quests.

## High-level flow

1. A user creates a category on Discord.
2. The category registration command enables auto recruitment for that category.
3. The bot creates the required channels (matching / date channels / quest channel) if it has permission.
4. Optionally, an operator adjusts the recruitment day range with a “days range” command.
5. Users register desired quests in the quest channel and their available times in date channels (order does not matter).
6. A periodic matching job (every 10 seconds) detects cases where 2+ users want the same date/time and quest.
7. On success, the bot posts to the matching channel and immediately creates a recruitment post in the same format as co-op recruitment v2.

## Layer dependency rules and implementation references

- Repository ports (traits) are defined under `src/repository/auto_recruitment/**`.
- Auto-recruitment concrete repositories are consolidated under `src/infrastructure/database/repositories/auto_recruitment/**`.
- `facades/auto_recruitment/**`, `services/auto_recruitment/**`, and `services/schedule/**` use trait-based dependencies via `crate::repository`.
- Concrete `SeaOrm*Repository` types are composed at DI/scheduler composition points (`src/di/repositories.rs`, `src/services/schedule/scheduler_manager.rs`).
- Do not introduce or depend on compatibility paths under `src/repository/database/**`.

### Implementation reference paths

```text
src/facades/auto_recruitment/
src/services/auto_recruitment/
src/services/schedule/auto_recruitment_rotation_task_executor.rs
src/services/schedule/auto_matching_task_executor.rs
src/services/schedule/scheduler_manager.rs
src/events/interactions/components/auto_recruit_time_handler.rs
src/repository/auto_recruitment/
src/infrastructure/database/repositories/auto_recruitment/
src/di/repositories.rs
```

## Required bot permissions

### Required
- Edit channel name

### Optional
- Create channels (if missing, users create channels manually)
- Delete channels (can be omitted)

## Slash commands

### Category registration command
- Registers the specified category for auto recruitment
- Day range is an optional parameter (2–7 days, default 7)
- If the bot has channel creation permission, it auto-creates the required channels
- If not, it completes registration and prompts the user to create channels manually

### Category unregistration command
- Disables auto recruitment for the specified category
- Deletes bot-created channels (matching / quest / date channels)
- For user-provided channels, deletes only the bot’s messages (keeps the channels)

### Day range change command
- Changes the number of days covered (2–7)
- If unset or default, it is 7 days

### Participation status command
- Shows your current registrations (which quests, which times)

### Register matching/quest channel command
- Registers a manually created matching channel or quest channel
- Specify channel type via parameter (matching / quest)

### Register date channels command
- Registers manually created date channels (up to 5 channels per command)
- The bot assigns dates automatically (fills missing dates first)
- After registration, if date channels are still insufficient, it informs the user
- Any channels beyond `days_range` are ignored (not stored in DB)

## Discord structure

### Category
- Prepared by the user

### Channel order
Channels inside the category are ordered as follows:

| Order | Channel | position |
|------:|---------|----------|
| 1 | Matching channel | 0 |
| 2 | Date channels (ascending) | 1..n |
| 3 | Quest selection channel | n+1 |

### Matching channel
- Exactly one channel in the category (position 0)
- On a successful match, posts a message showing date/time, quest, and participants
- Immediately creates a recruitment post in co-op recruitment v2 format
- Mentions all participants

### Date channels
- Any number of channels in the category (position 1..n)
- Channel name format: `M月d日` (e.g., `1月21日`)
- Sorted by date ascending
- Shows a select menu with 24 hourly options (supports selecting up to 24 at once)
- Selection is toggle-based (selecting again unregisters)

### Quest channel
- Exactly one channel in the category (position n+1)
- Uses “one message per quest”
- Quests without element/style selection: toggle join via a button
- 6-element quests: register/unregister via a multi-select menu (multiple selections allowed)
- Messages are generated from `master.quests` (ordered by `sort_order` descending)
- Adds a “Selected quests” button in the **last message**

#### “Selected quests” button
- Adds a dedicated message at the end of the quest channel (after all quest messages)
- Adds one button: “📋 Selected quests”
- On click, shows the user’s selections via an ephemeral message

##### Display example
```
📋 Your selected quests

🎮 Quest 1
🎮 Quest 2 (Fire, Water)
🎮 Quest 3 (Light)

Use the messages above to change your selections.
```

- If no quests are selected: show “No quests are selected.”
- For 6-element quests, also show the selected elements/styles.

#### In-game date handling (Granblue Fantasy)
- In GBF, the “day” changes at 5:00 AM. For example, the `1/21` channel covers `1/21 05:00` through `1/22 04:00`.
- Display hourly options in descending order (night hours first).
- Show in the order: next-day 04:00, 03:00, 02:00, 01:00, 00:00, then same-day 23:00 ... down to 05:00.
- For next-day hours (00:00–04:00), label as `翌0:00`, `翌1:00`, ... `翌4:00`.

### Shared channel rules
- Users cannot post or edit (bot-only)
- If the bot has channel creation permission, it creates channels automatically
- If not, users create channels manually and set appropriate permissions

## Date rotation

- Add a new internal scheduler task type and run it periodically
- Do not delete old date channels
- Rename outdated date channels to newer dates
- Reorder channels within the category (and update DB accordingly)

## Behavior on registering quests/times

- When a quest or time slot is registered, store it in the DB
- Search for match candidates based on the newly registered content
  - If a quest is changed: search using newly added quest(s)
  - If time availability is changed: search using newly added hour(s)
- If 2+ users exist for the same date/time and quest, a match succeeds

## Periodic matching process

### Scheduling
- Task type: `auto_matching`
- Interval: 10 seconds
- Concurrency: not allowed (register the next schedule after processing completes)

### Match detection
- Join `auto_recruitment_participants` and `user_desired_quests`
- Group by the same `(guild_id, quest_id, month, day, hour)`
- Extract groups with 2+ users
- Target only users not already registered in `quest_matchings`

### Grouping for 6-element quests
- If selected elements/styles overlap, split into separate groups
- If group size exceeds the participant cap (`recruit_count`), split into separate groups
- Place users with fewer selected elements first (prioritize assignment)

## On match success

### Notification
- Post a message in the matching channel
- Show date/time, quest, and participants
- Mention all participants
- For 6-element quests, also show assigned elements/styles

### Recruitment creation
- Immediately create a recruitment post in co-op recruitment v2 format
- Mention the quest-specific notification role (if any) and the matched participants

### Cancel
- Treat it as canceled if the scheduled time has passed

## DB tables

### Auto recruitment configuration (`guild_master.auto_recruitments`)
- guild_id
- category_id (category ID)
- matching_channel_id (matching channel ID)
- quest_channel_id (quest channel ID)
- matching_channel_is_bot_created (whether the matching channel was created by the bot)
- quest_channel_is_bot_created (whether the quest channel was created by the bot)
- matching_message_id (message ID posted in the matching channel)
- quest_message_id (deprecated: unused due to one-message-per-quest)
- days_range (number of days covered)
- created_at
- updated_at

### Auto recruitment date channels (`guild_master.auto_recruitment_channels`)
- guild_id
- channel_id (channel ID)
- month (target month)
- day (target day)
- sort_order (ordering)
- is_bot_created (whether the channel was created by the bot)
- message_id (posted message ID)
- created_at
- updated_at

### Quest message IDs (`guild_master.auto_recruitment_quest_messages`)
- guild_id (PK)
- quest_id (PK)
- message_id (message ID)
- created_at
- updated_at

### User desired quests (`guild_master.user_desired_quests`)
- guild_id (PK)
- user_id (PK, user ID)
- quest_id (PK, quest ID)
- battle_style_id (PK, desired element/style; 0=no specific selection)
- created_at
- updated_at

Notes:
- Primary key is `(guild_id, user_id, quest_id, battle_style_id)`.
- A user can request multiple elements/styles for the same quest.

### Time availability participants (`guild_master.auto_recruitment_participants`)
- guild_id
- user_id (user ID)
- month (target month)
- day (target day)
- hour (target hour)
- created_at
- updated_at

### Matchings (`worker.quest_matchings`)
- guild_id（PK）
- id (PK, UUID)
- quest_id (quest ID)
- scheduled_month (month)
- scheduled_day (day)
- scheduled_hour (hour)
- status (state: active/completed/cancelled)
- recruitment_id (created recruitment ID; set after creation)
- created_at
- updated_at

### Matching users (`worker.quest_matching_users`)
- guild_id（PK）
- matching_id (PK, FK → quest_matchings.id)
- user_id (PK, user ID)
- battle_style_id (assigned element/style; only for 6-element quests)
- joined_at (join timestamp)
- left_at (leave timestamp; NULL means still joined)

## Important rules

### Exclusion rule for date channels
- If a channel contains messages not authored by the bot, do not treat it as a date channel.
- This prevents accidentally deleting/losing user messages.

### Year boundary handling
- Multi-year schedules are not assumed.
- End-of-year adjustments are handled in a best-effort manner.

### Data retention
- Keep `user_desired_quests` and `auto_recruitment_participants` even after matching.
- Later, delete them in bulk via a cleanup job if needed.

### Data deletion on category unregistration
When the unregistration command is executed, delete data in this order:

1. Delete Discord channels/messages
2. `quest_matching_users` (delete first due to FK constraints)
3. `quest_matchings` (matching data)
4. `auto_recruitment_quest_messages` (quest message IDs)
5. `auto_recruitment_channels` (date channel info)
6. `auto_recruitments` (auto recruitment configuration)

## Error handling

| Situation | Handling |
|------|------|
| The channel exists in DB but was deleted on Discord | Notify error + recreate (if permitted) |
| No permission to access channel | Notify in the server’s default channel (future: dedicated error channel) |
| Attempted to register a date channel that contains non-bot messages | Abort registration + report to the user |
| Attempted to register more channels than `days_range` | Ignore excess (do not store in DB) |
| Not enough date channels | Report to the user |

## Related features

- [Co-op recruitment](multi_recruitment.md)
- [Scheduled recruitment](scheduled_recruitment.md)
