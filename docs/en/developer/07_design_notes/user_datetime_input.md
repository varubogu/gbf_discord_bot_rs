# Handling User Date/Time Input

## Purpose

This design note describes how to parse user input (e.g., `today 21:00`, `2200`, `2h before`) using consistent rules and safely convert it into the date/time types required by each feature.

## Background

Historically, separate parsers existed per use case, which caused:

- Non-unified interfaces
- Duplicated functionality and divergent rules
- Hard-to-understand impact scope when adding specs
- Scattered test coverage/concerns

Therefore, we move toward a design that handles input patterns in a unified way.

## Principles

- Explicitly list “allowed input patterns” per feature
- Distinguish parsed results into absolute datetime / relative time / time-of-day value as needed
- For multiple inputs (comma-separated), define per-use-case limits
- Classify errors at a granularity that can be returned to users

## Input feature flags (concept)

In the unified parser, input patterns are enabled/disabled per use case.

- `FULL_DATETIME`: full date + time
- `DATETIME_NO_YEAR`: date without year + time
- `DATE_ONLY`: date only
- `TIME_ONLY`: time only
- `JAPANESE_DATETIME`: Japanese date/time expressions
- `NUMERIC_PATTERNS`: numeric-only patterns (e.g., `2200`)
- `RELATIVE_TIME`: relative time expressions

## Allowed input formats

### Absolute date/time

- ISO-like formats such as `2024-12-31 21:00`
- No-year formats such as `12/31 21:00`
- Natural-language formats such as `Dec 31 9:30 PM`
- Relative day words + time such as `today 21:00` / `tomorrow 2200`
  - Both spaced and unspaced forms are allowed between the keyword and time (e.g., `tomorrow 21:00`, `tomorrow21:00`)

### Time only

- `21:00` format
- 4-digit numbers (e.g., `2200`)
- English time expressions (`9 PM`, `9:30 PM`)

### Relative time

- Examples: `2 hours ago`, `30 minutes later`, `1day`, `2h before`
- Units: day / hour / minute

## Constraints by use case

### Quest departure datetime

- Allowed: absolute datetime, no-year date, date-only, time-only, locale expressions, numeric patterns
- Disallowed: relative time (depending on feature requirements)

### Dismissal time(s)

- Allowed: absolute datetime + relative time
- Multiple values: comma-separated with an upper limit
- Reference time: departure datetime
- Relative expressions are limited to "before departure" (`later` / `after` are rejected)
- The allowed range from departure is restricted by `DISMISSAL_MAX_DAYS` (default `7` when unset)

### Scheduled recruitment start time

- Allowed: time-only + relative time
- Disallowed: specifying a date
- Reference time: quest start time

### Cases requiring strict `HH:MM`

- Allow `HH:MM` only
- Reject all ambiguous expressions

## Parsing flow (concept)

1. Check whether multiple values are allowed
2. If needed, split by comma
3. Parse each element
4. Determine strict mode
5. Parse absolute datetime
6. Parse relative time (only if allowed)
7. Normalize into typed results

## Result types

Depending on the use case, use:

- Absolute datetime (UTC)
- Relative time (delta from reference time: day/hour/minute)
- Time-of-day value (hour/minute only)

This separation allows scheduled recruitments, dismissals, and one-off recruitments to share the same input foundation.

## Direction rules for relative time

- `ago` / `before` means before the reference time
- `later` / `after` means after the reference time
- If direction is omitted, follow the target feature’s backward-compatibility policy

## Multiple values

- Only use cases with `allow_multiple=true` allow comma-separated values
- Ignore empty elements
- If the limit is exceeded, return an input error
- If any element fails to parse, fail the whole input (as required by the use case)

## Supported patterns (summary)

### Absolute patterns

- Full datetime: `2025/11/15 21:00`, `2025-11-15 21:00`
- No-year datetime: `12/11 14:00`, `12-11 14:00`
- Date only: `11/15`, `11-15`
- Time only: `21:00`, `9:30`
- Locale-friendly forms: `Jan 2 3:04`, `9:30 PM`, `March 16, 2026 21:00`
- Numeric: `1230`, `10111230`, `30 1230`, `202603162100`, `3/16 2100`, `0316 2100`, `03162100`

### Relative patterns

- Relative forms: `1day`, `2hours`, `90minutes`, `1h`, `90m`

## Error handling policy

- Invalid format (unsupported pattern)
- Out-of-range values (e.g., invalid date/time)
- Use-case constraint violations (disallowed input types)
- Count constraint violations (too many values)

Return short “what is wrong” messages to the UI, and keep details in logs.

## Testing notes

- Parsing per format (ISO/Japanese/English/numeric)
- Direction handling for relative time
- Year/month boundary cases
- Limits for multiple values, empty elements, mixed errors
- Allowed/denied boundaries per use case

## Operational notes

- When adding new input expressions, verify they do not change interpretations of existing features
- When specs change, also update user-facing input examples
- For locale-dependent expressions, explicitly document whether they are supported
