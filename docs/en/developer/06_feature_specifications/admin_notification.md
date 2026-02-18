# Admin Notification Feature

## Overview

The admin notification feature sends error messages and configuration warnings to
a designated Discord channel within each guild. Administrators holding the
`gbf_bot_control` role use this channel to monitor bot health.

## Channel type

- `channel_type = 5` (`GuildChannelType::AdminNotification` in `src/models/entities/master/channel_types.rs`)
- Registered per-guild in `guild_master.guild_channels`
- Set via `/channel_register` Discord slash command

## When notifications are sent

- A required channel (e.g. event-schedule notification channel) is not configured for a guild
- An error occurs during background task execution that should be reported to the guild admin
- Any other situation where the bot determines that the guild operator should be informed

## Notification behaviour when channel is not set

If `channel_type = 5` is not registered in `guild_master.guild_channels` for the
target guild, the service logs a `warn!` entry and returns `Ok(())` without
sending a Discord message.  This prevents an admin-notification failure from
cascading into the primary operation.

## Implementation

This feature is **service-only** (no dedicated facade).  The `AdminNotificationService`
is called from within other facades, which provide the surrounding transaction.

| Layer | File | Responsibility |
| --- | --- | --- |
| Enum | `src/models/entities/master/channel_types.rs` | `GuildChannelType::AdminNotification = 5` |
| Service | `src/services/channel/admin_notification_service.rs` | Fetch channel ID from DB, call Discord gateway |

> **No migration required.** The `master.channel_types` record for id=5 is managed
> outside this code (existing master data); no structural schema change is needed.

## Data flow

```
Caller facade (scheduler, etc.)
  ├─ begin transaction
  └─ AdminNotificationService::notify_admin(&txn, guild_id, message)
       ├─ GuildChannelRepository::get_by_guild_and_type_with_txn(guild_id, 5)
       │    └─ If None → warn log, return Ok(())
       └─ DiscordMessageGateway::send_message(channel_id, message)
```

## Related files

- `src/models/entities/master/channel_types.rs` – `GuildChannelType` enum definition
- `src/services/channel/admin_notification_service.rs` – service implementation (includes unit tests)
- `tests/integration/facades/admin_notification_test.rs` – integration tests
