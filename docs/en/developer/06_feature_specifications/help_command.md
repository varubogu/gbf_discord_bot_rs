# Help Command

## Overview

`/help` is an ephemeral command that shows available features as paged help content.  
Instead of a single fixed embed, it uses an index page plus page navigation UI.

## Goals

- Improve readability as features grow
- Show only commands relevant to the caller's permissions
- Centralize localized text in `locales/messages.yml`

## Command behavior

- Slash command: `/help`
- Response type: ephemeral
- Initial page: index (table of contents)

## UI behavior

### Index page

- Shows the list of pages visible to the caller
- Provides a direct-jump selector
- Provides `Back` / `Next` buttons

### Detail pages

- Show feature-specific command descriptions
- Provide `Back` / `Next` and `Back to Index`

### Navigation rule

- Loop navigation:
  - `Back` on the first page -> last page
  - `Next` on the last page -> first page

## Visibility by permission

### General user

- Index
- Multi Recruitment
- Scheduled Recruitment
- Auto Recruitment
- Utility / Reference

### `gbf_bot_control` holder

- All above + Admin Management

### Admin server (`BOT_ADMIN_SERVER_ID`) + `gbf_bot_control`

- All above + Bot Admin Server Only

## Component custom IDs

- Page navigation: `help_nav:{current_page}:{prev|next}`
- Jump selector: `help_nav:jump`
- Back to index: `help_nav:to_index`

Invalid custom IDs are safely ignored/deferred so the user experience is not broken.

## Localization and message keys

- New UI uses `help.navigator.*`
- Existing `help.embed.*` keys are kept for compatibility
- Japanese descriptions show `Japanese name / English name` for commands
- English descriptions show English command names only

## Implementation files

- `src/events/interactions/command_interactions/slash/util/help.rs`
- `src/events/interactions/help_navigator.rs`
- `src/events/interactions/components/help_navigator_handler.rs`
- `src/events/handlers/component_interaction.rs`
- `locales/messages.yml`
- `src/services/message/message_text_id.rs`
- `src/services/message/yaml_loader.rs`

## Test points

- Parse valid/invalid `help_nav` custom IDs
- Loop navigation (first page back / last page next)
- Permission-based visible page set
- Consistency between `MessageTextId` and `messages.yml`
