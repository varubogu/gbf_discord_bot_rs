# Troubleshooting (Operator)

## Common symptoms

### 1) Commands don’t respond

- Confirm the bot is running (process/container).
- Confirm Discord permissions are correctly configured.
- Confirm `DISCORD_TOKEN` is still valid.

### 2) Spreadsheet loading fails

- Confirm sharing settings (service account has editor permission).
- Confirm the spreadsheet ID/URL is correct.

### 3) Notifications don’t arrive

- Confirm the notification channel is registered.
- Confirm the bot’s scheduled processing is running (check logs).

## Check next

- [Monitoring and logs](07_monitoring_and_logs.md)
- [Start and update](10_start_and_update_docker_compose.md)
