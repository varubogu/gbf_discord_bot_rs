# Common Issues (Developer)

## The bot won’t start

- Confirm `DISCORD_TOKEN` is correct
- Confirm the DB is running and `DATABASE_URL` is correct
- Confirm the service account key JSON exists under `.local/`

## Spreadsheet-related failures

- Confirm sharing settings grant the service account “Editor” permission
- Confirm the spreadsheet ID/URL is correct (ID alone also works)

## DB-related test failures

- Confirm test DB prerequisites are satisfied (DB count, connection limits, migrations)
- Confirm you follow the integration test design notes (seed data / cleanup)

## Read next

- [Testing](../04_testing/README.md)
- [Database](../05_database/README.md)
