# Initial Setup Message

## Overview

This feature provides a guided setup message so that guild administrators can complete the required initial setup steps right after introducing the bot.
The same message content is shown through two paths:

- Immediately after the bot joins a guild (`GuildCreate.is_new = true`)
- When the `/init_message` command is executed (for re-display)

## Purpose

- Reduce setup omissions during first-time introduction (for example: spreadsheet not shared, notification channel not registered)
- Keep setup guidance inside the bot and reduce dependency on external operations docs
- Use one consistent guide message for both first join and manual re-display

## Message content

The initial setup message must include at least the following:

1. Open and copy the guild spreadsheet template URL
2. Grant edit permission on that spreadsheet to the bot service account
3. Register guild read/write spreadsheet URLs via `/gspread_register`
4. Import settings into DB via `/gspread_load`
5. Register notification channels via `/channel_register`

## Inputs (environment variables / files)

- `GUILD_SPREADSHEET_TEMPLATE_URL` (required)
  - Used as the template URL shown in the initial setup message
  - If unset, startup validation fails and blocks boot
- `GOOGLE_SERVICE_ACCOUNT_KEY_FILE` (existing)
  - Extract `client_email` from the JSON and show it as the sharing target address
  - If extraction fails, show a fallback text

## Behavior specification

### 1. On guild join

- Trigger: `FullEvent::GuildCreate`
- Conditions:
  - Send the initial setup message only when `is_new = true`
  - Do not send when `is_new = false` (guild re-received on bot restart)
- Destination:
  - Send only when `guild.system_channel_id` exists
  - If unset or send fails, log and continue (do not fail the whole event flow)

### 2. `/init_message` command

- Execution scope: guild only
- Permission: users with the `gbf_bot_control` role
- Response: send the initial setup message body into the current channel

## Layer responsibilities

- `events`
  - Build the initial setup message body
  - Control entry points for guild-join send and command send
- `facades/services/repository`
  - Keep responsibilities of existing `/gspread_register`, `/gspread_load`, and `/channel_register` unchanged

## Message management

- User-facing text is managed in `locales/messages.yml` (`messages.init_guide`)
- Code fetches text via `MessageTextId`
- Guild locale resolution follows existing `guild_settings.locale` rules

## Testing points

- String assembly:
  - Override / missing behavior of template URL environment variable
  - Success / failure when extracting `client_email` from service account JSON
- Message consistency:
  - Key consistency between `MessageTextId` and `locales/messages.yml`
  - `messages.init_guide` can be resolved by `yaml_loader`
