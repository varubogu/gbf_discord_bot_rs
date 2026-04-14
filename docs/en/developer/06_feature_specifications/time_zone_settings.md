# Server Settings (Timezone / Locale) — Design

## Overview

This feature configures timezone and locale (language setting) per Discord server (guild). Timezone settings make date/time input and display use the local time, and locale settings select the bot’s response language.

## Requirements

### Core features
- Configure timezone and locale via the slash command `/guild_settings_set`
- Specify timezone names compliant with the IANA Time Zone Database (e.g. `Asia/Tokyo`, `America/New_York`)
- Configure locale (e.g. `ja`, `en`)
- Defaults:
  - Timezone: `Asia/Tokyo`
  - Locale: `ja`
- Only users with the `gbf_bot_control` role can change settings

### Scope affected by timezone settings

#### Affected
1. **Interpretation of user input**
   - datetime input for `/recruit_new`
   - datetime input for `/recruit_change`
   - datetime interpretation when loading from spreadsheets

2. **Display**
   - event datetime shown in recruitment messages (with weekday)
   - event datetime shown in notification messages (with weekday)
   - schedule displays (with weekday)

#### Not affected
- Data stored in DB (always UTC)
- Internal calculations (always UTC)
- Recruitments created before changing the timezone (existing data is stored in UTC; conversion happens only at display time)

### Constraints
- Changes apply immediately
- Existing recruitment messages and DB records are not re-computed automatically
- Timezone is treated as the first setting that should be configured

## Architecture

### Responsibilities by layer

#### Presentation layer (`events/`)
```
src/events/interactions/command_interactions/slash/guild_settings/mod.rs
src/events/interactions/command_interactions/slash/guild_settings/guild_settings_set.rs
src/events/interactions/command_interactions/slash/guild_settings/guild_settings_show.rs
```
- Implement Discord API operations
- Define `/guild_settings_set`
- Authorization (`gbf_bot_control` role)
- Error handling
- Display the “settings updated” message

#### Facade layer (`facades/`)
```
src/facades/guild_settings/guild_settings_facade.rs
```
- Coordinate services
- Manage transaction boundaries
- Aggregate results

#### Service layer (`services/`)
```
src/services/timezone_service.rs
```
- Business logic for get/set timezone and locale
- Validate timezone name (parsable via `chrono-tz`)
- Validate locale
- Return defaults when unset

#### Repository port layer (`repository/`)
```
src/repository/guild_settings_repository.rs
```
- Define persistence contracts for guild settings (timezone / locale)
- Keep service-facing interfaces ORM-agnostic

#### Infrastructure adapter layer (`infrastructure/database/repositories/`)
```
src/infrastructure/database/repositories/guild/guild_settings_repository.rs
```
- Implement repository contracts with SeaORM
- Persist guild settings
- Find by guild ID
- UPSERT

### AppState dependency policy

- Build the default message service via DI (`src/di/message.rs`)
- `src/types/app_state.rs` stores `AppMessageService` and does not import repository concrete types directly

## Data model

### Primary entity

#### `GuildSettings`
```rust
pub struct Model {
    pub guild_id: i64,           // Discord Guild ID
    pub timezone: String,        // IANA timezone name
    pub locale: String,          // locale (language setting)
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**DB table**: `guild_settings`  
**Details**: [guild_settings.md](../05_database/schema/guild_master/guild_settings.md)

## Command spec

### `/guild_settings_set`

#### Parameters
| Parameter | Type | Required | Description | Examples |
|-----------|------|----------|-------------|----------|
| timezone | String | ⚪ | IANA timezone name (autocomplete supported) | `Asia/Tokyo`, `America/New_York`, `Europe/London` |
| locale | String | ⚪ | locale (language setting) | `ja`, `en` |

At least one of `timezone` or `locale` is required.

#### Autocomplete
- Show major timezone candidates while typing
- Display format: `timezone_name - description [UTC offset]`
  - Example: `Asia/Tokyo - Japan Standard Time (JST) [UTC+9]`
  - Example: `America/New_York - Eastern Standard Time (EST) [UTC-5]`
- Supports narrowing by substring
  - Timezone name (e.g. `tokyo`, `new_york`)
  - Description text (e.g. `japan`, `us`)
- Up to 25 items (Discord limit)
- Data source: an in-program array (~50 major timezones)

#### Flow
1. Authorization check (`gbf_bot_control` role)
2. Validate parameters
   - timezone validation (parsable via `chrono-tz`)
   - locale validation (supported locale)
3. Persist to DB (UPSERT)
4. Show completion message (ephemeral)

#### Success example
```
Updated server settings.
- Timezone: Asia/Tokyo
- Locale: ja
```

#### Error examples
- Permission denied: `This command can only be executed by administrators (users with the gbf_bot_control role).`
- Invalid timezone: `Invalid timezone name. Please specify an IANA timezone name (e.g. Asia/Tokyo, America/New_York).`
- Invalid locale: `Invalid locale. Supported locales: ja, en`
- Missing parameters: `Please specify either timezone or locale.`

## Check current settings

Current server settings can be checked by executing `/guild_settings_show` (or by implementing an equivalent dedicated “view settings” command).

#### Flow
1. Get current guild settings
2. Display timezone name and abbreviation

#### Example
```
Current timezone: Asia/Tokyo (JST)
Default settings: yes
```

Or when set explicitly:
```
Current timezone: America/New_York (EST)
Default settings: no
```

## Service API design

### `TimezoneService`

```rust
pub struct TimezoneService {
    repository: Arc<GuildTimezoneRepository>,
}

impl TimezoneService {
    /// Get the guild timezone
    /// Return default (Asia/Tokyo) when not configured
    pub async fn get_guild_timezone(&self, guild_id: GuildId) -> Result<Tz> {
        match self.repository.find_by_guild_id(guild_id).await? {
            Some(settings) => {
                settings.timezone.parse::<Tz>()
                    .map_err(|_| Error::InvalidTimezone)
            },
            None => {
                Ok(chrono_tz::Asia::Tokyo)
            }
        }
    }

    /// Set the guild timezone
    pub async fn set_guild_timezone(
        &self,
        guild_id: GuildId,
        timezone: Tz,
    ) -> Result<()> {
        // Accept a validated Tz and persist directly
        self.repository.upsert(guild_id, timezone.name()).await
    }

    /// Validate timezone name
    pub fn validate_timezone(timezone_str: &str) -> Result<Tz> {
        timezone_str.parse::<Tz>()
            .map_err(|_| Error::InvalidTimezone)
    }

    /// Get timezone list for autocomplete
    /// Filter by substring and show offset in UTC+9:00-style format
    /// Use cache for performance
    pub fn get_timezones_for_autocomplete(partial: &str) -> Vec<AutocompleteChoice> {
        // Partial match from all cached candidates in lazy_static
        // Display format: "Asia/Tokyo - Japan Standard Time (JST) [UTC+9]"
        // Return up to 25 items
    }

    /// Initialize timezone cache
    /// Call at startup to precompute values
    pub fn initialize_timezone_cache() {
        // Force lazy_static initialization
    }
}
```

### Major timezone list definition

```rust
/// List of major timezones (name, description)
/// Defined in-program as an array (~50 timezones)
const COMMON_TIMEZONES: &[(&str, &str)] = &[
    // Asia
    ("Asia/Tokyo", "Japan Standard Time (JST)"),
    ("Asia/Seoul", "Korea Standard Time (KST)"),
    ("Asia/Shanghai", "China Standard Time (CST)"),
    // ... other timezones
];

/// Cached timezone candidate list for autocomplete
/// Computed at startup and kept statically afterward
lazy_static! {
    static ref TIMEZONE_CHOICES: Vec<AutocompleteChoice> = {
        // Compute UTC offsets for all timezones
    };
}
```

### Performance optimization

The autocomplete timezone candidate list is cached with `lazy_static`:
- **At startup**: call `initialize_timezone_cache()` in `main.rs` to compute UTC offsets for all timezones
- **Afterwards**: fetch from the cached list and only perform substring filtering
- **Benefits**:
  - Offset computation cost (~50 timezone parses) is paid only at startup
  - No contention even with concurrent access
  - Memory-efficient (shared across all users)
- **Note**: DST offset changes are reflected on bot restart

## Repository API design

### `GuildTimezoneRepository`

```rust
pub struct GuildTimezoneRepository {
    db: Arc<DatabaseConnection>,
}

impl GuildTimezoneRepository {
    /// Find timezone setting by guild ID
    pub async fn find_by_guild_id(
        &self,
        guild_id: GuildId,
    ) -> Result<Option<guild_timezones::Model>> {
        guild_timezones::Entity::find_by_id(guild_id.get() as i64)
            .one(self.db.as_ref())
            .await
            .map_err(Into::into)
    }

    /// Save timezone setting (UPSERT)
    pub async fn upsert(
        &self,
        guild_id: GuildId,
        timezone: &str,
    ) -> Result<()> {
        let now = Utc::now();
        let model = guild_timezones::ActiveModel {
            guild_id: Set(guild_id.get() as i64),
            timezone: Set(timezone.to_string()),
            created_at: Set(now),
            updated_at: Set(now),
        };

        guild_timezones::Entity::insert(model)
            .on_conflict(
                OnConflict::column(guild_timezones::Column::GuildId)
                    .update_column(guild_timezones::Column::Timezone)
                    .update_column(guild_timezones::Column::UpdatedAt)
                    .to_owned()
            )
            .exec(self.db.as_ref())
            .await?;

        Ok(())
    }
}
```

## Error handling

### Error type definition

```rust
#[derive(Debug, thiserror::Error)]
pub enum TimezoneError {
    #[error("Invalid timezone name")]
    InvalidTimezone,

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Database error: {0}")]
    DatabaseError(#[from] DbErr),
}
```

## Test plan

### Unit tests

#### `TimezoneService`
- [ ] Validate a valid timezone name (success)
- [ ] Validate an invalid timezone name (failure)
- [ ] Return default timezone when unset
- [ ] Get a configured timezone

#### `GuildTimezoneRepository`
- [ ] UPSERT (insert)
- [ ] UPSERT (update)
- [ ] Find by guild ID (success)
- [ ] Find by non-existent guild ID (returns `None`)

### Integration tests

- [ ] Execute `/guild_settings_set` successfully
- [ ] Invalid timezone input in `/guild_settings_set`
- [ ] Permission failure in `/guild_settings_set`
- [ ] Execute `/guild_settings_show` (default state)
- [ ] Execute `/guild_settings_show` (configured state)
- [ ] Verify datetime interpretation in recruitment commands after setting the timezone

## Migration

### Migration file
```
migration/src/m20251208_000000_create_guild_timezones.rs
```

### DDL
```sql
CREATE TABLE guild_timezones (
    guild_id BIGINT PRIMARY KEY,
    timezone VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);
```

## Major timezone names (reference)

### Asia
| Timezone | Description | Abbrev | Standard UTC offset |
|----------|-------------|--------|---------------------|
| Asia/Tokyo | Japan Standard Time | JST | UTC+9 |
| Asia/Seoul | Korea Standard Time | KST | UTC+9 |
| Asia/Shanghai | China Standard Time | CST | UTC+8 |
| Asia/Hong_Kong | Hong Kong Time | HKT | UTC+8 |
| Asia/Taipei | Taipei Standard Time | CST | UTC+8 |
| Asia/Singapore | Singapore Standard Time | SGT | UTC+8 |
| Asia/Bangkok | Indochina Time | ICT | UTC+7 |
| Asia/Jakarta | Western Indonesian Time | WIB | UTC+7 |
| Asia/Manila | Philippine Standard Time | PST | UTC+8 |
| Asia/Kolkata | India Standard Time | IST | UTC+5:30 |
| Asia/Dubai | Gulf Standard Time | GST | UTC+4 |

### Oceania
| Timezone | Description | Abbrev | Standard UTC offset |
|----------|-------------|--------|---------------------|
| Australia/Sydney | Australian Eastern Standard Time | AEST | UTC+10 |
| Australia/Melbourne | Australian Eastern Standard Time | AEST | UTC+10 |
| Australia/Perth | Australian Western Standard Time | AWST | UTC+8 |
| Pacific/Auckland | New Zealand Standard Time | NZST | UTC+12 |

### North America
| Timezone | Description | Abbrev | Standard UTC offset |
|----------|-------------|--------|---------------------|
| America/New_York | Eastern Standard Time | EST | UTC-5 |
| America/Chicago | Central Standard Time | CST | UTC-6 |
| America/Denver | Mountain Standard Time | MST | UTC-7 |
| America/Los_Angeles | Pacific Standard Time | PST | UTC-8 |
| America/Anchorage | Alaska Standard Time | AKST | UTC-9 |
| America/Toronto | Eastern Standard Time | EST | UTC-5 |
| America/Vancouver | Pacific Standard Time | PST | UTC-8 |

### Central & South America
| Timezone | Description | Abbrev | Standard UTC offset |
|----------|-------------|--------|---------------------|
| America/Mexico_City | Central Standard Time (Mexico) | CST | UTC-6 |
| America/Sao_Paulo | Brasilia Time | BRT | UTC-3 |
| America/Buenos_Aires | Argentina Time | ART | UTC-3 |

### Europe
| Timezone | Description | Abbrev | Standard UTC offset |
|----------|-------------|--------|---------------------|
| Europe/London | Greenwich Mean Time | GMT | UTC+0 |
| Europe/Paris | Central European Time | CET | UTC+1 |
| Europe/Berlin | Central European Time | CET | UTC+1 |
| Europe/Rome | Central European Time | CET | UTC+1 |
| Europe/Madrid | Central European Time | CET | UTC+1 |
| Europe/Moscow | Moscow Standard Time | MSK | UTC+3 |

### Africa
| Timezone | Description | Abbrev | Standard UTC offset |
|----------|-------------|--------|---------------------|
| Africa/Cairo | Eastern European Time | EET | UTC+2 |
| Africa/Johannesburg | South Africa Standard Time | SAST | UTC+2 |

### Other
| Timezone | Description | Abbrev | Standard UTC offset |
|----------|-------------|--------|---------------------|
| UTC | Coordinated Universal Time | UTC | UTC+0 |

**Notes:**
- Regions observing DST (daylight saving time) may have offsets different from the standard value
- Shown UTC offsets are computed based on the current time
- ~50 timezones are defined in-program

## Implementation checklist

Design phase:
- [x] Define requirements
- [x] Design architecture
- [x] Design data model
- [x] Define command spec
- [x] Design error handling
- [x] Test plan

Implementation phase:
- [ ] Add `chrono-tz` to Cargo.toml
- [ ] Create migration file
- [ ] Define entity (`guild_timezones::Model`)
- [ ] Implement repository
- [ ] Implement service
- [ ] Implement facade
- [ ] Implement command (`/guild_settings_set`)
- [ ] Implement command (`/guild_settings_show`)
- [ ] Update `datetime_parser` service
- [ ] Update display logic
- [ ] Update spreadsheet loading
- [ ] Add tests
