# Feature Specifications (Developer)

This section summarizes each major feature’s purpose, requirements, data model, and processing flow.

## Index

- [スプレッドシート連携](スプレッドシート連携.md)
- [マルチ募集](マルチ募集.md)
- [定期募集](定期募集.md)
- [募集通知](募集通知.md)
- [自動募集](自動募集.md)
- [スケジュール機能](スケジュール機能.md)
- [タイムゾーン設定](タイムゾーン設定.md)
- [起動時検証](起動時検証.md)

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
