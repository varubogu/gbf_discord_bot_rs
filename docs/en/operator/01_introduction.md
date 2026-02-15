# Introduction (Bot Operator)

## Purpose

This page helps you understand the overall picture required to operate the bot.
While this operator documentation is not as deep as developer documentation, it assumes a basic level of IT literacy.
In particular, you should understand folders/files, Docker basics, and environment variables.

## What to remember (minimum)

- The bot depends on **Discord**, **PostgreSQL (database)**, and **Google Sheets**.
- Even if the bot process stops, DB data usually remains (but you still need backups).
- Configuration is split into **environment variables** and **spreadsheets (global / per-server)**.

## Roles and responsibilities

### Bot operator (infra/deploy)

- Deploy/update the bot (prefer container/CI artifacts if possible)
- DB backup/restore, migrations
- Monitoring, log checks, incident response
- Manage the global spreadsheet

### Discord server administrator (Guild Master)

Server-side configuration (roles, channels, spreadsheets, etc.) is documented separately.

- [Server administrator documentation](../server_administrator_guild_master/README.md)

## Read next

- [Installation](02_installation_overview.md)
- [Configuration (environment variables)](03_installation_configuration_details.md)
