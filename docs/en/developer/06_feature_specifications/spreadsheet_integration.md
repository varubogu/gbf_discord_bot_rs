# Google Sheets Integration

## Overview

This feature enables bi-directional data synchronization between Google Sheets and a PostgreSQL database. Master data and guild-specific settings can be managed in a spreadsheet and synchronized with the bot.

## Goals

- **Simplify data management**: manage data in spreadsheets without writing SQL
- **Editable by non-engineers**: change bot settings using standard spreadsheet operations
- **Backup and versioning**: leverage Google Drive version history
- **Visualize data**: view and edit data in a tabular UI

## Feature categories

This integration provides two variants:

### 1. Global spreadsheet integration

**Use**: manage master data shared across all guilds

- **Target data**: `battle_types`, `quests`, `elements`, `messages`, etc.
- **Who can run**: only in the bot admin-only server
- **Impact scope**: all guilds
- **Commands**:
  - `/gspread_global_load` - Google Sheets → PostgreSQL
  - `/gspread_global_push` - PostgreSQL → Google Sheets

### 2. Guild spreadsheet integration

**Use**: manage custom data per guild

- **Target data**: `guild_messages`, `guild_event_schedules`, etc. (guild-specific data) + spreadsheet configuration tables (`guild_spreadsheet_imports` / `guild_spreadsheet_exports`)
- **Additional auto-recruitment setting data**: `auto_recruitment_match_rules`, `auto_recruitment_match_rule_quotas`
- **Who can run**: users with the `gbf_bot_control` role
- **Impact scope**: only the guild where the command is executed
- **Commands**:
  - `/gspread_regist` - register spreadsheets for a guild
  - `/gspread_load` - Google Sheets → PostgreSQL
  - `/gspread_push` - PostgreSQL → Google Sheets

## Primary use cases

### Use case 1: Bulk register quest data

**Scenario**: a new multi-battle quest is added

1. A bot administrator edits the “Quest Info” sheet in the global spreadsheet
2. Add the new quest details (name, recruitment count, default strategy, etc.)
3. Execute `/gspread_global_load`
4. The new quest becomes available in all guilds

### Use case 2: Configure a guild-specific event schedule

**Scenario**: a guild wants to use its own event notification schedule

1. A guild administrator edits the “Guild Event Schedule” sheet in the guild spreadsheet
2. Configure notification timing and target channels
3. Execute `/gspread_load`
4. The custom schedule becomes active only for that guild

### Use case 3: Customize message templates

**Scenario**: a guild wants to adjust bot message wording to match its culture

1. A guild administrator edits the “Guild Message Definitions” sheet in the guild spreadsheet
2. Change the text for a specific message ID
3. Execute `/gspread_load`
4. Customized messages are displayed only in that guild

### Use case 4: Backup data

**Scenario**: save the current database state to a spreadsheet

1. Execute `/gspread_global_push` or `/gspread_push`
2. PostgreSQL data is written to Google Sheets
3. Google Drive version history keeps the backup

### Use case 5: Register spreadsheets during initial guild setup

**Scenario**: a newly onboarded guild wants to start spreadsheet integration

1. A guild administrator creates a Google Sheet and grants the service account view access
2. Register the spreadsheet URL via `/gspread_regist`
3. The spreadsheet IDs are saved to `guild_spreadsheet_imports` and `guild_spreadsheet_exports`
4. Synchronization becomes available via `/gspread_load` and `/gspread_push`

### Use case 6: Auto-generate UUID IDs

**Scenario**: when adding a new quest row in a spreadsheet, you do not want to manually input the ID

1. A bot administrator adds a new row in the “Quest Info” sheet of the global spreadsheet
2. Leave the `quest_id` column (UUID) **empty**, and fill in other fields (name, recruit count, etc.)
3. Execute `/gspread_global_load`
4. The bot generates a new UUID and inserts it into the database
5. **The generated UUID is also written back to the spreadsheet**
6. On the next load, the same UUID is used, preserving consistency

**Benefits**:
- No need to copy/paste UUIDs manually
- Spreadsheet IDs and database IDs always match
- Zero risk of primary key duplication

### Use case 7: Configure guild-specific auto-recruitment matching

**Scenario**: one guild wants different matching requirements for the same quest without restarting the bot

1. A guild administrator edits `auto_recruitment_match_rules` and, when needed, `auto_recruitment_match_rule_quotas`
2. The guild administrator executes `/gspread_load`
3. The bot validates preset names, quest IDs, element compatibility, and quota totals inside the import transaction
4. The next periodic auto-matching run uses the updated rule immediately

## Spreadsheet structure

### `table_names` sheet (metadata)

Every spreadsheet has a special metadata sheet named `table_names`, which defines which tables are synchronized.

| Row/Col | Content | Notes |
|--------:|---------|-------|
| Row 1 | mapping keys such as `sheet_name`, `table_name`, `table_scope`, `table_io`, `table_type` | use `snake_case` |
| Row 2 | Japanese column descriptions | not used by the program |
| Row 3+ | table definition rows | rows with missing required fields are skipped |

- `table_io`: `"in"`, `"out"`, `"in,out"` (or `"out,in"` / `"both"`)
- `table_type`: `"reference"`, `"transaction"`, `"history"`
- `table_scope`: for future expansion (can be empty if unused)
- Unknown keys are ignored, so you can add columns flexibly
- Column order does not matter (mapped by key name)

**Meaning of `table_io`**:
- `in` - Google Sheets → PostgreSQL (read/import only)
- `out` - PostgreSQL → Google Sheets (write/export only)
- `in,out` - bi-directional

### Per-table sheets

Create one sheet per table.

**Sheet layout**:
- **Row 1**: physical column names (must match PostgreSQL exactly; mapped by key name)
- **Row 2**: Japanese column names (optional; unused by the program)
- **Row 3+**: data rows (if you use description rows, place data from row 3 onward)

Column order does not matter; the program maps by the column names in row 1. Undefined columns are ignored automatically.

**Example: `quests` sheet**

| target_id | recruit_count | quest_name | use_battle_type | default_battle_type |
|-----------|--------------|-----------|----------------|-------------------|
| Quest ID | Recruit Count | Quest Name | Available Strategies | Default Strategy |
| 1 | 30 | Proto Bahamut HL | 1,2,3 | 1 |
| 2 | 18 | Ultimate Bahamut HL | 1,2 | 2 |

**Example: `auto_recruitment_match_rules` sheet**

| guild_id | quest_id | preset_type | min_match_count | required_battle_style_id | required_battle_style_count |
|----------|----------|-------------|-----------------|--------------------------|-----------------------------|
| Guild ID | Quest ID | Preset Type | Minimum Match Count | Required Element | Required Element Count |
| 12345 | 10 | min_members_only | 4 |  |  |
| 12345 | 20 | specific_element_n_plus_any | 4 | 1 | 2 |

**Example: `auto_recruitment_match_rule_quotas` sheet**

| guild_id | quest_id | battle_style_id | required_count | sort_order |
|----------|----------|-----------------|----------------|------------|
| Guild ID | Quest ID | Element ID | Required Count | Sort Order |
| 12345 | 30 | 1 | 1 | 10 |
| 12345 | 30 | 2 | 1 | 20 |

## Data lookup priority

If both guild-specific data and global data exist, **guild-specific data takes precedence**.

```
Lookup priority when reading data:
1. Search guild-specific tables (`guild_*`)
2. If not found, search global tables
3. If not found anywhere, use defaults or return an error
```

**Example: get a message**
- Guild A defines `welcome_message` in `guild_messages` → show the guild-specific message
- Guild B does not define it → show the global `messages.welcome_message`

## Permission model

### Global spreadsheets

- **Who can run**: members of the bot admin-only server
- **Environment variable**: admin server is identified by `BOT_ADMIN_SERVER_ID`
- **Response**: regular message (visible to other admins)

### Guild spreadsheets

- **Who can run**: users with the `gbf_bot_control` role
- **Response**: regular message (default)

## `/gspread_regist` command design

This command registers the spreadsheet IDs (import/export) for a guild into dedicated tables (`guild_spreadsheet_imports`, `guild_spreadsheet_exports`) as a prerequisite for `/gspread_load` and `/gspread_push`.

### Goals

- Enable spreadsheet integration via UI only, even right after installing the bot
- Store a guild-specific alternative to `GLOBAL_SPREADSHEET_ID` in the DB to avoid exploding env vars
- Detect spreadsheet permission issues early and surface them before execution

### Command spec

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `load_spreadsheet_url` | String (max 512 chars) | ✅ | Spreadsheet used for importing guild data. Accepts an ID-only input; after normalization, save to `guild_spreadsheet_imports`. |
| `push_spreadsheet_url` | String (max 512 chars) | ✅ | Spreadsheet used for exporting guild data. If using the same sheet, pass the same value as `load_spreadsheet_url`. |

### Flow

1. **Authorization**: verify the actor has the `gbf_bot_control` role
2. **Get `guild_id`**: read the guild ID from the command context (DM is not allowed)
3. **Normalize URLs**:
   - Normalize both URL/ID inputs to `https://docs.google.com/spreadsheets/d/{id}`
   - Validate `spreadsheet_id` with regex `^[A-Za-z0-9-_]{20,80}$`
4. **Permission check**:
   - For each `spreadsheet_id`, call Google Sheets API `spreadsheets.get`
   - If 403/404, guide the user to fix sharing settings
5. **Facade** (`GuildSpreadsheetRegistrationFacade`):
   - Begin a transaction via `TransactionManager`
   - `GuildSpreadsheetConfigService` executes `upsert(guild_id, spreadsheet_id)` on both `guild_spreadsheet_imports` and `guild_spreadsheet_exports`
   - To support “update only one side”, the facade detects argument differences and updates only what is needed
   - After commit, generate a response including normalized URLs
6. **Response**:
   - Success: display `✅ Registered guild spreadsheets` and both URLs
   - Failure: convert to a user-facing message via `PresentationError`

### Validation & error cases

- **Invalid URL format**: `❌ Error: Invalid Google Sheets URL format`
- **Insufficient sharing permissions**: `❌ Error: The service account does not have view access. Please check the spreadsheet sharing settings.`
- **DB write failure**: rollback immediately (no retry), respond `❌ Failed to save guild settings`
- **Concurrent execution**: naturally serialized by PK(`guild_id`), and idempotency is ensured by `INSERT ... ON CONFLICT` in the facade

### Persistence

- Table: `guild_spreadsheet_imports` (import)
- Table: `guild_spreadsheet_exports` (export)
- Both use `guild_id` as PK and store `spreadsheet_id` as the value (minimal schema)

## Repository structure and transaction responsibility

- Repository trait (port): `src/repository/guild_spreadsheet_config_repository.rs`
- SeaORM adapter (implementation): `src/infrastructure/database/repositories/guild/guild_spreadsheet_config_repository.rs`
- Facades instantiate the infrastructure adapter, while services depend on the trait contract.
- `upsert_*` operations must receive a transaction that is started/committed/rolled back by the Facade layer.
- Do not use legacy direct DB wrappers (`models_database`, `db_compat`) from spreadsheet features.

### Relation to other commands

- `/gspread_load` requires `guild_spreadsheet_imports`; if missing, it prompts users to run `/gspread_regist`
- `/gspread_push` requires `guild_spreadsheet_exports`; if missing, it returns an error guiding users to register
- In the future, add a view command so admins can check the registered values in both tables

### Auto-recruitment rule validation on `/gspread_load`

- `preset_type` must be one of `min_members_only`, `one_each_element`, `specific_element_n_plus_any`, `fixed_element_quota`
- `quest_id` must resolve to an existing quest
- `min_match_count` must be at least `2`
- Attribute-based presets are allowed only for six-element quests
- `specific_element_n_plus_any` requires both `required_battle_style_id` and `required_battle_style_count`
- `fixed_element_quota` requires at least one quota row and the sum of `required_count` must match `min_match_count`
- Validation runs inside the import transaction, so invalid rules roll back the entire `/gspread_load`

## Authentication

Use OAuth2 with a Google Cloud Platform (GCP) service account.

**Required environment variables**:
```bash
GOOGLE_SERVICE_ACCOUNT_KEY_FILE=/path/to/service-account-key.json
```

**Required Google scope**:
- `https://www.googleapis.com/auth/spreadsheets`

## Error handling

### Behavior on errors

- **Spreadsheet connection failure**: abort command execution and show an error message
- **Data conversion errors**: skip error rows, log details, continue processing
- **Database errors**: rollback the transaction and discard all changes

### Example user-facing messages

```
❌ Error: This command can only be executed in the bot admin-only server

❌ Error: Failed to connect to Google Sheets
Please check logs for details

❌ Error: An error occurred during data conversion
- quests row 5: recruit_count must be a number
```

## Performance considerations

- **Bulk insert**: insert in batches instead of row-by-row
- **Parallel processing**: process independent tables concurrently
- **Transactions**: run all table updates in a single transaction (all-or-nothing)

## Security

- **Service account key**: protect with filesystem permissions (600 recommended)
- **Spreadsheet URLs**: manage via env vars; do not print to logs
- **SQL injection**: prevented by SeaORM parameter binding
- **Mask secrets**: do not include sensitive data in error logs

## Constraints

### Current constraints

- **Full sync only**: no delta updates (future enhancement)
- **No real-time sync**: synchronization occurs only when commands are executed
- **Spreadsheet count limit**: 1 global spreadsheet; 1 per guild

### Recommended operations

- **Small datasets**: recommend up to a few thousand rows
- **Periodic backups**: regularly export via `/gspread_push`
- **Review change history**: use Google Drive version history

## Related documents

### Architecture
- [layered_architecture.md](../02_architecture/layered_architecture.md)
- [dependency_injection.md](../02_architecture/dependency_injection.md)

### Database
- [overview.md](../05_database/overview.md)
- [connections_and_transactions.md](../05_database/connections_and_transactions.md)
- [guild_spreadsheet_imports.md](../05_database/schema/guild_master/guild_spreadsheet_imports.md)
- [guild_spreadsheet_exports.md](../05_database/schema/guild_master/guild_spreadsheet_exports.md)

### Rules
- [error_handling.md](../03_development_rules/error_handling.md)
- [logging.md](../03_development_rules/logging.md)
- [security.md](../03_development_rules/security.md)

## Data conversion spec (unified)

### Goal

Define the spec for mutual conversion between Google Sheets (strings) and DB types, managed as part of this feature spec.

### Basic principles

- Prefer type-safety; values incompatible with DB types must become explicit errors
- Handle conversion errors per row; allow normal rows to continue processing
- Define empty-string behavior (NULL/default) per type
- Use consistent rules for auto-completed fields such as UUID

### PostgreSQL → Google Sheets conversion

| DB type | Output | Notes |
| --- | --- | --- |
| INTEGER / BIGINT | numeric string | e.g. `123` |
| TEXT / VARCHAR | string as-is | escaping is delegated to the API layer |
| BOOLEAN | `true` / `false` | lowercase |
| TIMESTAMPTZ | RFC3339 string | keep timezone info |
| UUID | UUID string | hyphenated standard form |
| NULL | empty string | represented as an empty cell |

### Google Sheets → PostgreSQL conversion

#### Numbers

- Target: INTEGER / BIGINT
- Allowed: numeric strings
- Invalid: non-numeric, overflow
- Empty string: follow column definition (NULLability / default)

#### Strings

- Target: TEXT / VARCHAR
- Empty + NOT NULL: required-field error
- Empty + NULL allowed: convert to NULL

#### Booleans

- Allowed: `true/false`, `1/0`, `yes/no`, `t/f`
- Invalid: type conversion error
- Empty string handling: follow column definition / default

#### Date/time

- Primary: RFC3339
- Secondary: `YYYY-MM-DD HH:MM:SS`, `YYYY-MM-DD`
- Invalid: date/time format error

#### UUID

- Accept valid UUID strings as-is
- If a UUID column is empty and the row is considered “new”, auto-generate a UUID

### UUID auto-generation and write-back

- On import, detect empty UUID cells and generate UUIDs before inserting into DB
- Write generated UUIDs back to the spreadsheet to preserve identity on the next import
- If write-back fails, rollback the DB transaction to preserve consistency

### Error-handling policy

- Row-level conversion errors: skip the row and aggregate counts/details
- Fatal errors (API outage, transaction failure): treat the entire run as failed
- Show a summary to users and log details

### Validation policy

- Type validation (number/boolean/datetime/UUID)
- Required-field validation (NOT NULL)
- Foreign-key consistency (presence of referenced entities)
- For guild data, validate `guild_id` consistency

### Consistency during sync

- Imports run inside a transaction to preserve global consistency
- When replacing per table, preserve ordering constraints (referential integrity)
- Avoid concurrent runs via operational rules

### Test considerations

- Normal/abnormal conversions per type
- Boundaries for NULL / empty string / defaults
- UUID auto-generation and rollback on write-back failure
- Date/time format variations
- Error aggregation and performance for large datasets

### Type-specific conversion examples

| Input | Expected type | Result | Notes |
| --- | --- | --- | --- |
| `123` | INTEGER | OK | accept as numeric |
| `9999999999` | INTEGER | NG | out-of-range |
| (empty) | NOT NULL string | NG | required |
| (empty) | nullable string | OK | convert to NULL |
| `true` | BOOLEAN | OK | true |
| `abc` | BOOLEAN | NG | type conversion error |
| `2025-01-15T12:00:00+09:00` | TIMESTAMPTZ | OK | RFC3339 |
| `2025-01-15 12:00:00` | TIMESTAMPTZ | OK | secondary format |
| `not-a-date` | TIMESTAMPTZ | NG | format error |

### Datetime format priority

1. RFC3339
2. `YYYY-MM-DD HH:MM:SS`
3. `YYYY-MM-DD` (time is filled by a default rule)

When changing implementation, keep this priority to preserve compatibility with existing inputs.

### Conversion error classes (ops perspective)

- `TypeConversionError`: type mismatch (number/boolean/UUID, etc.)
- `RequiredFieldMissing`: required field missing
- `ValueOutOfRange`: out of range
- `DateTimeFormatError`: invalid datetime format

### Handling by error class

- Recoverable, row-level: skip row + aggregate
- Unrecoverable (external connection / transaction): rollback all

## FAQ

### Q1: When should I use the global spreadsheet vs the guild spreadsheet?

**A**: Use the global spreadsheet for master data shared across all guilds (quest info, strategy definitions, etc.). Use the guild spreadsheet for per-guild customization (message text, event schedules, etc.).

### Q2: What happens to rows that failed conversion?

**A**: Failed rows are skipped and details are logged. Other valid rows are still processed. The user sees the number of failed rows and a summary of errors.

### Q3: When does the transaction commit?

**A**: It commits after successfully applying data for all tables. If an error occurs mid-way, all changes are rolled back.

### Q4: Where are spreadsheet URLs managed?

**A**: The global spreadsheet is managed via environment variables. Guild spreadsheets are stored in dedicated tables (`guild_spreadsheet_imports` / `guild_spreadsheet_exports`), and environment variables are used only as fallback.

### Q5: Is existing data overwritten?

**A**: Yes. When `/gspread_load` runs, the target tables are deleted and replaced with spreadsheet data (TRUNCATE + INSERT).

### Q6: What happens if multiple people run it at the same time?

**A**: Transactions provide mutual exclusion, but the final result depends on execution order (last writer wins). It is recommended to avoid concurrent runs via operational rules.

### Q7: What happens if a UUID ID cell is empty?

**A**: The Rust side generates a new UUID (UUIDv4) and inserts it into the database. The generated UUID is also written back to the spreadsheet, so the same UUID is used on the next import.

### Q8: What happens if UUID write-back fails?

**A**: The database insert is rolled back and an error is shown. This preserves consistency between spreadsheet and database and prevents PK duplication on the next import.
