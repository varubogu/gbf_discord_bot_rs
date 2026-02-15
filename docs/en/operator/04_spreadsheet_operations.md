# Spreadsheet Operations (Bot Operator)

The bot uses Google Sheets to manage data such as “quest information”, “environment variables”, and “messages”.
This page focuses on what bot operators manage (more on the global side), and summarizes operational practices and minimum pitfalls.

## What to remember (minimum)

- **Global**: Data shared across all servers (managed by bot operators)
- **Per-server (guild)**: Data for a specific Discord server (managed by server admins)

For per-server (guild) steps, see the separate guide.

- [Server administrator documentation](../server_administrator_guild_master/README.md)

## Common tasks

### 1) Register a spreadsheet

- Tell the bot which spreadsheet to use.
- If it fails, re-check the URL/ID and sharing settings (permissions).

### 2) Load a spreadsheet

- Apply spreadsheet → DB.
- Run this during initial setup and after changing settings.

### 3) Write back to a spreadsheet

- Export DB → spreadsheet (e.g., statistics).

## Common failure points

- Forgetting to grant the service account **Editor** permission.
- Missing required sheet tabs or having incorrect sheet names.

## If you’re stuck

- First, confirm the sharing settings and the ID/URL you entered.
- Then, check both the load command result message and the bot logs.
