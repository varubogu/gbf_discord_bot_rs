# Startup Validation

## Overview

This feature checks required environment variables and files at application startup and displays clear messages if something is missing or invalid. It prevents misconfiguration from becoming runtime failures and improves the experience for developers and operators.

## Goals

- **Early error detection**: catch configuration issues at startup and prevent runtime errors
- **Clear feedback**: make it obvious which setting is problematic
- **Operational efficiency**: reduce troubleshooting time
- **Better developer experience**: reduce trial-and-error during first-time setup

## Functional requirements

### 1. Startup environment checks

At application startup (before DB connection), check the following:

#### Check items

**Required environment variables (shared)**:
- `DB_HOST` - DB host
- `DB_PORT` - DB port
- `DB_NAME` - DB name

**Required environment variables (normal bot startup only)**:
- `DISCORD_TOKEN` - Discord bot token
- `BOT_ADMIN_SERVER_ID` - Admin-only server ID for bot operators
- `GUILD_DB_USER` - Guild role username
- `GUILD_DB_PASSWORD` - Guild role password
- `SYSTEM_DB_USER` - System role username
- `SYSTEM_DB_PASSWORD` - System role password
- `GLOBAL_DB_USER` - Global role username
- `GLOBAL_DB_PASSWORD` - Global role password
- `GUILD_SPREADSHEET_TEMPLATE_URL` - Template spreadsheet URL shown in guild init messages

**Required environment variables (migration execution)**:
- `ADMIN_DB_USER` - Admin role username
- `ADMIN_DB_PASSWORD` - Admin role password

> Note: normal startup runs migrations in the startup sequence, so normal startup validates all three groups (`shared + normal bot startup + migration execution`). `migrate-only` validates only `shared + migration execution`.

**Optional environment variables**:
- `GLOBAL_SPREADSHEET_ID` - Global spreadsheet ID (required only when spreadsheet features are used)
- `GOOGLE_SERVICE_ACCOUNT_KEY_FILE` - Path to Google service account key file (required only when spreadsheet features are used)

**File checks**:
- Confirm the file specified by `GOOGLE_SERVICE_ACCOUNT_KEY_FILE` exists
- If it exists, confirm it can be read as valid JSON

### 2. Output format

```
🔍 Starting environment validation...
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Required Environment Variables:
  DISCORD_TOKEN........................✅ OK
  BOT_ADMIN_SERVER_ID..................✅ OK
  DB_HOST..............................✅ OK
  DB_PORT..............................✅ OK
  DB_NAME..............................✅ OK
  GUILD_DB_USER........................✅ OK
  GUILD_DB_PASSWORD....................✅ OK
  SYSTEM_DB_USER.......................✅ OK
  SYSTEM_DB_PASSWORD...................✅ OK
  GLOBAL_DB_USER.......................✅ OK
  GLOBAL_DB_PASSWORD...................✅ OK
  ADMIN_DB_USER........................✅ OK
  ADMIN_DB_PASSWORD....................✅ OK

Optional Environment Variables:
  GLOBAL_SPREADSHEET_ID................❌ NOT SET (required for spreadsheet features)
  GOOGLE_SERVICE_ACCOUNT_KEY_FILE......✅ OK

File Validation:
  Service Account Key File.............✅ OK (valid JSON)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Validation Result: ❌ FAILED (1 error, 0 warnings)

❌ Errors:
  - GLOBAL_SPREADSHEET_ID is not set but required for spreadsheet features

💡 Next Steps:
  1. Set GLOBAL_SPREADSHEET_ID in your .env.app file
  2. Restart the application

Exiting...
```

**On success**:
```
🔍 Starting environment validation...
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Required Environment Variables:
  DISCORD_TOKEN........................✅ OK
  BOT_ADMIN_SERVER_ID..................✅ OK
  DB_HOST..............................✅ OK
  DB_PORT..............................✅ OK
  DB_NAME..............................✅ OK
  GUILD_DB_USER........................✅ OK
  GUILD_DB_PASSWORD....................✅ OK
  SYSTEM_DB_USER.......................✅ OK
  SYSTEM_DB_PASSWORD...................✅ OK
  GLOBAL_DB_USER.......................✅ OK
  GLOBAL_DB_PASSWORD...................✅ OK
  ADMIN_DB_USER........................✅ OK
  ADMIN_DB_PASSWORD....................✅ OK

Optional Environment Variables:
  GLOBAL_SPREADSHEET_ID................✅ OK
  GOOGLE_SERVICE_ACCOUNT_KEY_FILE......✅ OK

File Validation:
  Service Account Key File.............✅ OK (valid JSON)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Validation Result: ✅ PASSED

Proceeding with application startup...
```

### 3. Detailed runtime error display

For errors such as DB connection failures or JSON parse failures, display detailed context:

```
❌ Database Connection Error

Environment Variables: DB_HOST, DB_PORT, DB_NAME, GUILD_DB_USER, GUILD_DB_PASSWORD
Constructed URL: postgresql://guild_user:***@localhost:5432/invalid_db
Error Details: Connection refused (SQLSTATE: 08001)

Possible Causes:
  - PostgreSQL server is not running
  - Database name is incorrect
  - Host or port is unreachable
  - Authentication failed

💡 Troubleshooting:
  1. Check PostgreSQL server status: sudo systemctl status postgresql
  2. Verify DB connection parameters: DB_HOST, DB_PORT, DB_NAME
  3. Verify role credentials: GUILD_DB_USER, GUILD_DB_PASSWORD
  4. Test connection: psql "postgresql://guild_user:password@localhost:5432/gbf_bot_db"
```

```
❌ JSON Parse Error

Environment Variable: GOOGLE_SERVICE_ACCOUNT_KEY_FILE
File Path: /path/to/service-account-key.json
Error Details: EOF while parsing a value at line 1, column 0

Possible Causes:
  - File is empty
  - File contains invalid JSON
  - File encoding is incorrect

💡 Troubleshooting:
  1. Check file contents: cat /path/to/service-account-key.json
  2. Validate JSON format: jq . /path/to/service-account-key.json
  3. Re-download service account key from Google Cloud Console
```

## Non-functional requirements

### Performance
- Environment checks should complete within 1 second
- File reading should be asynchronous

### Usability
- Error messages shown to end users should be in Japanese
- Use visually clear symbols (✅❌⚠️💡)
- For long values (tokens, etc.), display only the prefix/suffix (e.g., `dMzY...xy2A`)

### Security
- Mask secrets (tokens, passwords) in output
- Do not emit secrets into log files

## Architecture

### Components

```
main.rs
  ↓
StartupValidator (new)
  ↓
EnvValidator (new)
  ├─ check_required_env_vars()
  ├─ check_optional_env_vars()
  └─ check_file_validation()
```

### Startup/environment DB access policy

- Startup validation itself is limited to environment variables and file checks, and must not depend on legacy wrappers.
- If environment values are loaded from DB, use `src/infrastructure/database/session/DatabaseSession` (temporary compatibility adapter) or an `AppState`-managed connection.
- Do not directly use `models_database` / `db_compat`.
- Session-context SQL (for example RLS context variables) must be handled via `src/infrastructure/database/session`.

### Types

#### StartupValidator

```rust
/// Startup validator
pub struct StartupValidator {
    results: Vec<ValidationResult>,
}

impl StartupValidator {
    pub fn new() -> Self;

    /// Run validation checks for the startup mode
    pub async fn validate_for_mode(mode: StartupValidationMode) -> Result<Self, StartupError>;

    /// Display validation results
    pub fn display_results(&self);

    /// Whether validation succeeded
    pub fn is_valid(&self) -> bool;
}
```

#### ValidationResult

```rust
/// Validation result
pub struct ValidationResult {
    pub category: ValidationCategory,
    pub item_name: String,
    pub status: ValidationStatus,
    pub message: Option<String>,
    pub help_text: Option<String>,
}

pub enum ValidationCategory {
    RequiredEnvVar,
    OptionalEnvVar,
    FileValidation,
    DatabaseConnection,
}

pub enum ValidationStatus {
    Ok,
    Warning,
    Error,
}
```

#### EnvValidator

```rust
/// Environment variable validator
pub struct EnvValidator;

impl EnvValidator {
    /// Check required environment variables common to all modes
    pub fn check_common_required_vars() -> Vec<ValidationResult>;

    /// Check required environment variables for normal bot startup
    pub fn check_bot_startup_required_vars() -> Vec<ValidationResult>;

    /// Check required environment variables for migration mode
    pub fn check_migration_required_vars() -> Vec<ValidationResult>;

    /// Check optional environment variables
    pub fn check_optional_vars() -> Vec<ValidationResult>;

    /// Validate file existence and contents
    pub async fn check_files() -> Vec<ValidationResult>;

    /// Validate a JSON file
    pub async fn validate_json_file(path: &str) -> Result<(), String>;
}
```

#### ErrorFormatter (new)

```rust
/// Error formatter
pub struct ErrorFormatter;

impl ErrorFormatter {
    /// Format a database error
    pub fn format_db_error(err: &DbErr, db_url: &str) -> String;

    /// Format a JSON parse error
    pub fn format_json_error(err: &serde_json::Error, file_path: &str) -> String;

    /// Format a generic error
    pub fn format_generic_error(err: &dyn std::error::Error, context: &str) -> String;
}
```

### Error type

```rust
/// Startup validation error
#[derive(Error, Debug)]
pub enum StartupError {
    #[error("Required environment variable not set: {var_name}")]
    MissingRequiredEnvVar { var_name: String },

    #[error("File not found: {file_path} (specified in {env_var})")]
    FileNotFound { file_path: String, env_var: String },

    #[error("Invalid JSON in file: {file_path}\n{details}")]
    InvalidJson { file_path: String, details: String },

    #[error("Database connection failed: {details}")]
    DatabaseConnectionFailed { details: String },

    #[error("Multiple validation errors occurred")]
    MultipleErrors { errors: Vec<String> },
}
```

## Implementation flow

### Startup sequence

```
1. main()
   ↓
2. StartupValidator::validate_for_mode(mode)
   ↓
3. EnvValidator::check_common_required_vars()
   ├─ DB_HOST
   ├─ DB_PORT
   ├─ DB_NAME
   ↓
4. Mode branch
   ├─ NormalStartup
   │  ├─ EnvValidator::check_bot_startup_required_vars()
   │  ├─ EnvValidator::check_migration_required_vars()
   │  ├─ EnvValidator::check_optional_vars()
   │  └─ EnvValidator::check_files()
   └─ MigrationOnly
      └─ EnvValidator::check_migration_required_vars()
   ↓
5. StartupValidator::display_results()
   ↓
6. StartupValidator::is_valid() ?
   ├─ Yes → Continue to database initialization
   └─ No → Exit with code 1
```

### Error handling flow

```
Runtime Error Occurs
   ↓
Capture Error with Context
   ↓
ErrorFormatter::format_*_error()
   ↓
Display Formatted Error
   ↓
Exit with appropriate code
```

## Configuration reference

### Required environment variables (shared)

| Variable | Description | Example | Validation |
|-------|------|-------|------------|
| `DB_HOST` | DB host | `localhost` | Non-empty string |
| `DB_PORT` | DB port | `5432` | Numeric; 1–65535 |
| `DB_NAME` | DB name | `gbf_bot_db` | Non-empty string |

### Required environment variables (normal bot startup only)

| Variable | Description | Example | Validation |
|-------|------|-------|------------|
| `DISCORD_TOKEN` | Discord bot token | `MTIzNDU2Nzg5...` | Non-empty; 30+ chars |
| `BOT_ADMIN_SERVER_ID` | Admin server ID | `123456789012345678` | Numeric; 18–20 digits |
| `GUILD_DB_USER` | Guild role username | `guild_user` | Non-empty string |
| `GUILD_DB_PASSWORD` | Guild role password | `********` | Non-empty string |
| `SYSTEM_DB_USER` | System role username | `system_user` | Non-empty string |
| `SYSTEM_DB_PASSWORD` | System role password | `********` | Non-empty string |
| `GLOBAL_DB_USER` | Global role username | `global_user` | Non-empty string |
| `GLOBAL_DB_PASSWORD` | Global role password | `********` | Non-empty string |
| `GUILD_SPREADSHEET_TEMPLATE_URL` | Guild init message template spreadsheet URL | `https://docs.google.com/spreadsheets/d/{id}/copy` | Google spreadsheet URL format |

### Required environment variables (migration execution)

| Variable | Description | Example | Validation |
|-------|------|-------|------------|
| `ADMIN_DB_USER` | Admin role username | `admin_user` | Non-empty string |
| `ADMIN_DB_PASSWORD` | Admin role password | `********` | Non-empty string |

### Optional environment variables

| Variable | Description | Example | Validation |
|-------|------|-------|------------|
| `GLOBAL_SPREADSHEET_ID` | Global spreadsheet ID | `1BxiMVs0XRA5...` | 20–80 alphanumeric chars |
| `GOOGLE_SERVICE_ACCOUNT_KEY_FILE` | Service account key file path | `/path/to/key.json` | File exists and readable |

### File validation

| File | Source | Validation |
|---------|--------|------------|
| Service Account Key | `GOOGLE_SERVICE_ACCOUNT_KEY_FILE` | Exists; valid JSON; required fields (`type`, `project_id`, `private_key`) |

## Constraints

### Current constraints

- Env var checks are synchronous (fast enough)
- File checks are asynchronous (supports large files)
- DB connection tests are not performed (to avoid impacting startup time)

### Future extensions

- Add optional DB connectivity tests
- Add optional Discord API connectivity tests
- Validate configuration files (e.g., `.env.app` / `.env.db` syntax checks)

## Test strategy

### Unit tests

- `EnvValidator::check_common_required_vars()` / `check_bot_startup_required_vars()` / `check_migration_required_vars()` - presence checks
- `EnvValidator::validate_json_file()` - JSON validation
- `ErrorFormatter::format_*_error()` - message formatting

### Integration tests

- Behavior when env vars are missing
- Behavior when JSON files are invalid
- Output verification when all checks pass

### Manual tests

- Verify startup in a real environment
- Validate readability of error messages
- Validate troubleshooting steps

## References

- [Twelve-Factor App: Config](https://12factor.net/config)
- [Rust Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [tracing - Structured Logging](https://docs.rs/tracing/latest/tracing/)
