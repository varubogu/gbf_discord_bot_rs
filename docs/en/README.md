# docs (New Documentation)

This `docs/` directory reorganizes the repository documentation by the reader’s role.

> Note: `docs/en` is currently a work in progress. Some pages may still be in Japanese.

## Choose your role

- [Developer](開発者向け/README.md)
- [Bot operator](運用者向け/README.md)
- [User](利用者向け/README.md)
- [Server administrator (Guild Master)](サーバー管理者（ギルドマスター）向け/README.md)

## What “operator” means in this repo

There are two main kinds of “operators”:

- **Bot operator (infrastructure/deploy)**: Responsible for uptime, DB, backups, updates, monitoring, and incident response.
- **Discord server administrator (Guild Master)**: Responsible for configuring the bot inside a specific Discord server (channels, spreadsheets, roles, etc.).

## Writing guidelines for docs

- Write in English (assume non-experts may read it).
- Structure: conclusion → steps → common pitfalls (FAQ).
- Do not explain specs by pasting code; focus on concepts, responsibilities, and flow.
- Always link to “where to go next”.
