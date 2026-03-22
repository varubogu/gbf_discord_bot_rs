# Feature Specifications (Developer)

This section summarizes each major feature's purpose, requirements, data model, and processing flow.

## Index

- [Spreadsheet integration](spreadsheet_integration.md)
- [Co-op recruitment](multi_recruitment.md)
- [Scheduled recruitment](scheduled_recruitment.md)
- [Help command](help_command.md)
- [Recruitment notifications](recruitment_notifications.md)
- [Auto recruitment](auto_recruitment.md)
- [Scheduling feature (common)](scheduling_feature.md)
- [Scheduling feature (by task type)](scheduling_feature/README.md)
- [Message resolution](message_resolution.md)
- [Time zone settings](time_zone_settings.md)
- [Startup validation](startup_validation.md)
- [Admin notification](admin_notification.md)

## Feature permissions by role

| Feature | General user | Discord server admin | Bot operator (admin) |
| --- | --- | --- | --- |
| Read/write per-server spreadsheet | x | o | o |
| Read/write global spreadsheet | x | x | o |
| Create/join co-op recruitment | o | o | o |
| Edit/delete other users' co-op recruitments | x | o | o |
| Join scheduled recruitment | o | o | o |
| Create/edit/delete scheduled recruitment | o | o | o |
| Quest list | o | o | o |
| Quest enable/disable | x | o | o |
| View recruitment roles | o | o | o |
| Configure recruitment roles | x | o | o |
| View server channel settings | o | o | o |
| Edit server channel settings | x | o | o |
| View server settings | o | o | o |
| Edit server settings | x | o | o |
| Recruitment notifications | o | o | o |
| Auto recruitment | o | o | o |
| Schedule notification feature | o | o | o |
| Schedule editing feature | x | o | o |
| Time zone settings | x | o | o |

## Notes

- This section is a reorganized version based on `docs/develop/features`.
- For DB details, see [05_database](../05_database).
- For implementation rules, see [03_development_rules](../03_development_rules).
