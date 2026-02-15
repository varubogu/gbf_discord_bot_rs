# Troubleshooting (Server Administrator)

## Common symptoms

### 1) Commands don’t respond

- Check that the bot has permission to view/post in that channel.
- For admin-only actions, confirm the command user has the `gbf_bot_control` role.
- Contact the bot operator to confirm the bot process/container is running.

### 2) Spreadsheet loading fails

- Confirm sharing settings (service account has editor permission).
- Confirm the spreadsheet ID/URL is correct.
- Confirm required sheet tabs exist and names match.

### 3) Notifications don’t arrive

- Confirm the notification channel is registered.
- Confirm the bot has permission to post in that channel.
- Confirm server settings (enabled/disabled, etc.) are as intended.

## If it looks like a bot-side issue

- [Bot operator documentation](../operator/README.md)
