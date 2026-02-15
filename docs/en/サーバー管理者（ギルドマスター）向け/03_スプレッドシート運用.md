# Spreadsheet Operations (Server Administrator)

The bot uses Google Sheets to manage server-specific settings.
This page focuses on what server administrators handle (per-server / guild spreadsheets), and summarizes the minimum concepts and pitfalls.

## What to remember (minimum)

- **Global**: Data shared across all servers (managed by the bot operator).
- **Per-server (guild)**: Data for a specific Discord server (managed by server admins).

## Common tasks

### 1) Register a spreadsheet

- Tell the bot which spreadsheet to use.
- If it fails, re-check the URL/ID and the sharing settings (permissions).

### 2) Load a spreadsheet

- Apply spreadsheet → DB.
- Run this during initial setup and after changing settings.

### 3) Write back to a spreadsheet

- Export DB → spreadsheet (e.g., statistics).

## Common failure points

- Forgetting to grant the service account **Editor** permission.
- Missing required sheet tabs or having incorrect sheet names.
- Forgetting to “load” after changing settings, so changes don’t apply.

## If you’re stuck

- First, confirm the sharing settings and the ID/URL you entered.
- Then, check the result message from the load command.
- If it still doesn’t resolve, ask the bot operator to check the logs.
