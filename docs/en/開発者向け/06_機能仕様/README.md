# Feature Specifications (Developer)

This section summarizes each major feature’s purpose, requirements, data model, and processing flow.

## Index

- [Spreadsheet integration](スプレッドシート連携.md)
- [Co-op recruitment](マルチ募集.md)
- [Scheduled recruitment](定期募集.md)
- [Recruitment notifications](募集通知.md)
- [Auto recruitment](自動募集.md)
- [Scheduling platform](スケジュール機能.md)
- [Time zone settings](タイムゾーン設定.md)
- [Startup validation](起動時検証.md)

## Feature permissions by role

| Feature | General user | Discord server admin | Bot operator (admin) |
| --- | --- | --- | --- |
| Read/write per-server spreadsheet | ✕ | ◯ | ◯ |
| Read/write global spreadsheet | ✕ | ✕ | ◯ |
| Create/join co-op recruitment | ◯ | ◯ | ◯ |
| Edit/delete other users’ co-op recruitments | ✕ | ◯ | ◯ |
| Join scheduled recruitment | ◯ | ◯ | ◯ |
| Create/edit/delete scheduled recruitment | ◯ | ◯ | ◯ |
| Quest list | ◯ | ◯ | ◯ |
| Quest enable/disable | ✕ | ◯ | ◯ |
| View recruitment roles | ◯ | ◯ | ◯ |
| Configure recruitment roles | ✕ | ◯ | ◯ |
| View server channel settings | ◯ | ◯ | ◯ |
| Edit server channel settings | ✕ | ◯ | ◯ |
| View server settings | ◯ | ◯ | ◯ |
| Edit server settings | ✕ | ◯ | ◯ |
| Recruitment notifications | ◯ | ◯ | ◯ |
| Auto recruitment | ◯ | ◯ | ◯ |
| Schedule notification feature | ◯ | ◯ | ◯ |
| Schedule editing feature | ✕ | ◯ | ◯ |
| Time zone settings | ✕ | ◯ | ◯ |

## Notes

- This section is a reorganized version based on `docs/develop/features`.
- For DB details, see [05_データベース](../05_データベース).
- For implementation rules, see [03_開発ルール](../03_開発ルール).
