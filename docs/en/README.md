# docs (New Documentation)

This `docs/` directory reorganizes the repository documentation by the reader’s role.

> Note: `docs/en` is maintained as the English counterpart of `docs/ja`. Keep both sides synchronized when updating specs.

## Choose your role

- [Developer](developer/README.md)
- [Bot operator](operator/README.md)
- [User](user/README.md)
- [Server administrator (Guild Master)](server_administrator_guild_master/README.md)

## What “operator” means in this repo

There are two main kinds of “operators”:

- **Bot operator (infrastructure/deploy)**: Responsible for uptime, DB, backups, updates, monitoring, and incident response.
- **Discord server administrator (Guild Master)**: Responsible for configuring the bot inside a specific Discord server (channels, spreadsheets, roles, etc.).

## Writing guidelines for docs

- Write in English (assume non-experts may read it).
- Structure: conclusion → steps → common pitfalls (FAQ).
- Do not explain specs by pasting code; focus on concepts, responsibilities, and flow.
- Always link to “where to go next”.
