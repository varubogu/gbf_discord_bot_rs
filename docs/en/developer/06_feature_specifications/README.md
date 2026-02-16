# Feature Specifications (Developer)

This section summarizes each major feature’s purpose, requirements, data model, and processing flow.

## Index

- [Spreadsheet integration](spreadsheet_integration.md)
- [Co-op recruitment](multi_recruitment.md)
- [Scheduled recruitment](scheduled_recruitment.md)
- [Recruitment notifications](recruitment_notifications.md)
- [Auto recruitment](auto_recruitment.md)
- [Scheduling platform](scheduling_feature.md)
- [Message resolution](message_resolution.md)
- [Time zone settings](time_zone_settings.md)
- [Startup validation](startup_validation.md)

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
- For DB details, see [05_database](../05_database).
- For implementation rules, see [03_development_rules](../03_development_rules).
