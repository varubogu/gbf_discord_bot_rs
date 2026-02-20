# Message Resolution

## Overview

This feature defines the shared rules for resolving an actual message body from a `message_text_id`.
Each feature (for example, schedule notifications, recruitment, and settings display) delegates body resolution to `MessageService`.

## Goals

- Unify where message texts are managed
- Enable guild-specific customization
- Prevent feature outages by falling back to `locales/messages.yml` when DB records are missing

## Target data sources

1. `guild_master.guild_message_texts`
2. `master.message_texts`
3. `locales/messages.yml`

## Resolution priority

Body resolution for a `message_text_id` is performed in this order.

1. Guild message texts (`guild_message_texts`)
2. Global message texts (`message_texts`)
3. `locales/messages.yml`
4. If none of 1-3 resolves, treat as an error

### Notes

- Even if a DB access error occurs at 1 or 2, do not stop processing; continue to the next fallback
- 3 is the final fallback, and if resolved there, message sending continues

## Locale resolution

Language selection for message text is based on the locale provided by the caller.

- `ja` locale family: use Japanese text
- Non-`ja` locales: prefer English text; if English is undefined, fall back to Japanese text

### Locale in guild-scoped features

For guild-scoped responses, locale is resolved from `guild_settings.locale`.
If unset, `ja` is used.

- `MessageService` keeps its internal fallback behavior (`locale=None` -> `en`).
- Therefore, guild-scoped callers must not call `MessageService` with `locale=None`.
- Callers must resolve `guild_settings.locale` first and pass it explicitly (fallback to `ja` when unset).

### Locale in schedule notifications

Schedule notifications follow the same rule above and resolve locale from `guild_settings.locale`.
If unset, `ja` is used.

## Parameter substitution

`{{key}}` placeholders in message text are replaced by parameters provided by the caller.

- Example: `Hello {{user_name}}`
- Behavior: replace only when the corresponding key exists

## Error policy

- Missing data at 1 or 2: do not treat as an error; continue fallback
- DB error at 1 or 2: log at warn level and continue fallback
- Unresolved after 1-3: treat as business error equivalent to `MessageTextNotResolved`

## Relation to scheduling feature

- The scheduling feature first determines `message_text_id` per notification row
- Body resolution (`guild` -> `global` -> `messages.yml` -> error) follows this specification
- For details, see [Scheduling platform](./scheduling_feature.md)

## Implementation sources

- `src/services/message/message_service.rs`
- `src/services/message/message_text_id.rs`
- `src/repository/guild_message_text_repository.rs`
- `src/repository/message_text_repository.rs`
- `src/infrastructure/database/repositories/guild/guild_message_text_repository.rs`
- `src/infrastructure/database/repositories/master_data/message_text_repository.rs`
- `src/di/message.rs`
- `src/types/app_state.rs`
- `src/services/locale_service.rs`
- `src/events/helpers.rs`
- `locales/messages.yml`

## Related documents

- [Scheduling platform](./scheduling_feature.md)
- [Coding standards](../03_development_rules/coding_standards.md)
- [master.message_texts](../05_database/schema/master/message_texts.md)
- [guild_master.guild_message_texts](../05_database/schema/guild_master/guild_message_texts.md)
