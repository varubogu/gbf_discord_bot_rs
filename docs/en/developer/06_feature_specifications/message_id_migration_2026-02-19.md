# Message ID Migration (2026-02-19)

## Overview

To remove duplicate definitions in `locales/messages.yml`, message IDs are fully renamed.
This migration does **not** provide a runtime compatibility layer for old IDs.

- Old IDs become unresolved in code
- IDs in DB and spreadsheets must be replaced operationally

## ID mapping (before / after)

| Old ID | New ID |
| --- | --- |
| `schedule.command.generate.success_field_name` | `schedule.command.shared.success_field_name` |
| `schedule.command.global_generate.success_field_name` | `schedule.command.shared.success_field_name` |
| `schedule.command.generate.success_footer` | `schedule.command.shared.success_footer` |
| `schedule.command.global_generate.success_footer` | `schedule.command.shared.success_footer` |
| `schedule.command.generate.error_footer` | `schedule.command.shared.error_footer` |
| `schedule.command.global_generate.error_footer` | `schedule.command.shared.error_footer` |
| `recruitment.command.change.panel_quest_unchanged` | `recruitment.command.change.panel_unchanged` |
| `recruitment.command.change.panel_style_unchanged` | `recruitment.command.change.panel_unchanged` |
| `recruitment.command.change.panel_date_unchanged` | `recruitment.command.change.panel_unchanged` |
| `recruitment.command.change.error_prefix` | `common.error_prefix` |
| `auto_recruitment.operation_error_prefix` | `common.error_prefix` |

## Priority rules when values conflict

Multiple old IDs are consolidated into one new ID.
If values conflict during replacement, apply this priority order:

1. `schedule.command.shared.success_field_name`:
   prioritize `schedule.command.generate.success_field_name`
2. `schedule.command.shared.success_footer`:
   prioritize `schedule.command.generate.success_footer`
3. `schedule.command.shared.error_footer`:
   prioritize `schedule.command.generate.error_footer`
4. `recruitment.command.change.panel_unchanged`:
   prioritize `recruitment.command.change.panel_quest_unchanged`
5. `common.error_prefix`:
   prioritize `recruitment.command.change.error_prefix`

Notes:

- In current default definitions, those source values are identical, so results stay the same when no customizations exist.
- Guild-specific custom values may differ, so this fixed priority is required.

## DB replacement procedure

Targets:

- `master.message_texts`
- `guild_master.guild_message_texts`

### 1. Pre-check (count old IDs)

```sql
SELECT id, COUNT(*) AS count
FROM master.message_texts
WHERE id IN (
  'schedule.command.generate.success_field_name',
  'schedule.command.global_generate.success_field_name',
  'schedule.command.generate.success_footer',
  'schedule.command.global_generate.success_footer',
  'schedule.command.generate.error_footer',
  'schedule.command.global_generate.error_footer',
  'recruitment.command.change.panel_quest_unchanged',
  'recruitment.command.change.panel_style_unchanged',
  'recruitment.command.change.panel_date_unchanged',
  'recruitment.command.change.error_prefix',
  'auto_recruitment.operation_error_prefix'
)
GROUP BY id
ORDER BY id;
```

```sql
SELECT guild_id, id, COUNT(*) AS count
FROM guild_master.guild_message_texts
WHERE id IN (
  'schedule.command.generate.success_field_name',
  'schedule.command.global_generate.success_field_name',
  'schedule.command.generate.success_footer',
  'schedule.command.global_generate.success_footer',
  'schedule.command.generate.error_footer',
  'schedule.command.global_generate.error_footer',
  'recruitment.command.change.panel_quest_unchanged',
  'recruitment.command.change.panel_style_unchanged',
  'recruitment.command.change.panel_date_unchanged',
  'recruitment.command.change.error_prefix',
  'auto_recruitment.operation_error_prefix'
)
GROUP BY guild_id, id
ORDER BY guild_id, id;
```

### 2. Create consolidated IDs (apply priority rules)

Example for `master.message_texts`:

```sql
BEGIN;

INSERT INTO master.message_texts (id, message_jp, message_en, created_at, updated_at)
SELECT
  'schedule.command.shared.success_field_name',
  src.message_jp,
  src.message_en,
  NOW(),
  NOW()
FROM master.message_texts src
WHERE src.id = 'schedule.command.generate.success_field_name'
ON CONFLICT (id) DO UPDATE
SET
  message_jp = EXCLUDED.message_jp,
  message_en = EXCLUDED.message_en,
  updated_at = NOW();

INSERT INTO master.message_texts (id, message_jp, message_en, created_at, updated_at)
SELECT
  'schedule.command.shared.success_footer',
  src.message_jp,
  src.message_en,
  NOW(),
  NOW()
FROM master.message_texts src
WHERE src.id = 'schedule.command.generate.success_footer'
ON CONFLICT (id) DO UPDATE
SET
  message_jp = EXCLUDED.message_jp,
  message_en = EXCLUDED.message_en,
  updated_at = NOW();

INSERT INTO master.message_texts (id, message_jp, message_en, created_at, updated_at)
SELECT
  'schedule.command.shared.error_footer',
  src.message_jp,
  src.message_en,
  NOW(),
  NOW()
FROM master.message_texts src
WHERE src.id = 'schedule.command.generate.error_footer'
ON CONFLICT (id) DO UPDATE
SET
  message_jp = EXCLUDED.message_jp,
  message_en = EXCLUDED.message_en,
  updated_at = NOW();

INSERT INTO master.message_texts (id, message_jp, message_en, created_at, updated_at)
SELECT
  'recruitment.command.change.panel_unchanged',
  src.message_jp,
  src.message_en,
  NOW(),
  NOW()
FROM master.message_texts src
WHERE src.id = 'recruitment.command.change.panel_quest_unchanged'
ON CONFLICT (id) DO UPDATE
SET
  message_jp = EXCLUDED.message_jp,
  message_en = EXCLUDED.message_en,
  updated_at = NOW();

INSERT INTO master.message_texts (id, message_jp, message_en, created_at, updated_at)
SELECT
  'common.error_prefix',
  src.message_jp,
  src.message_en,
  NOW(),
  NOW()
FROM master.message_texts src
WHERE src.id = 'recruitment.command.change.error_prefix'
ON CONFLICT (id) DO UPDATE
SET
  message_jp = EXCLUDED.message_jp,
  message_en = EXCLUDED.message_en,
  updated_at = NOW();

COMMIT;
```

Example for `guild_master.guild_message_texts`:

```sql
BEGIN;

INSERT INTO guild_master.guild_message_texts (guild_id, id, message_jp, message_en, created_at, updated_at)
SELECT
  src.guild_id,
  'schedule.command.shared.success_field_name',
  src.message_jp,
  src.message_en,
  NOW(),
  NOW()
FROM guild_master.guild_message_texts src
WHERE src.id = 'schedule.command.generate.success_field_name'
ON CONFLICT (guild_id, id) DO UPDATE
SET
  message_jp = EXCLUDED.message_jp,
  message_en = EXCLUDED.message_en,
  updated_at = NOW();

INSERT INTO guild_master.guild_message_texts (guild_id, id, message_jp, message_en, created_at, updated_at)
SELECT
  src.guild_id,
  'schedule.command.shared.success_footer',
  src.message_jp,
  src.message_en,
  NOW(),
  NOW()
FROM guild_master.guild_message_texts src
WHERE src.id = 'schedule.command.generate.success_footer'
ON CONFLICT (guild_id, id) DO UPDATE
SET
  message_jp = EXCLUDED.message_jp,
  message_en = EXCLUDED.message_en,
  updated_at = NOW();

INSERT INTO guild_master.guild_message_texts (guild_id, id, message_jp, message_en, created_at, updated_at)
SELECT
  src.guild_id,
  'schedule.command.shared.error_footer',
  src.message_jp,
  src.message_en,
  NOW(),
  NOW()
FROM guild_master.guild_message_texts src
WHERE src.id = 'schedule.command.generate.error_footer'
ON CONFLICT (guild_id, id) DO UPDATE
SET
  message_jp = EXCLUDED.message_jp,
  message_en = EXCLUDED.message_en,
  updated_at = NOW();

INSERT INTO guild_master.guild_message_texts (guild_id, id, message_jp, message_en, created_at, updated_at)
SELECT
  src.guild_id,
  'recruitment.command.change.panel_unchanged',
  src.message_jp,
  src.message_en,
  NOW(),
  NOW()
FROM guild_master.guild_message_texts src
WHERE src.id = 'recruitment.command.change.panel_quest_unchanged'
ON CONFLICT (guild_id, id) DO UPDATE
SET
  message_jp = EXCLUDED.message_jp,
  message_en = EXCLUDED.message_en,
  updated_at = NOW();

INSERT INTO guild_master.guild_message_texts (guild_id, id, message_jp, message_en, created_at, updated_at)
SELECT
  src.guild_id,
  'common.error_prefix',
  src.message_jp,
  src.message_en,
  NOW(),
  NOW()
FROM guild_master.guild_message_texts src
WHERE src.id = 'recruitment.command.change.error_prefix'
ON CONFLICT (guild_id, id) DO UPDATE
SET
  message_jp = EXCLUDED.message_jp,
  message_en = EXCLUDED.message_en,
  updated_at = NOW();

COMMIT;
```

### 3. Delete old IDs

```sql
DELETE FROM master.message_texts
WHERE id IN (
  'schedule.command.generate.success_field_name',
  'schedule.command.global_generate.success_field_name',
  'schedule.command.generate.success_footer',
  'schedule.command.global_generate.success_footer',
  'schedule.command.generate.error_footer',
  'schedule.command.global_generate.error_footer',
  'recruitment.command.change.panel_quest_unchanged',
  'recruitment.command.change.panel_style_unchanged',
  'recruitment.command.change.panel_date_unchanged',
  'recruitment.command.change.error_prefix',
  'auto_recruitment.operation_error_prefix'
);
```

```sql
DELETE FROM guild_master.guild_message_texts
WHERE id IN (
  'schedule.command.generate.success_field_name',
  'schedule.command.global_generate.success_field_name',
  'schedule.command.generate.success_footer',
  'schedule.command.global_generate.success_footer',
  'schedule.command.generate.error_footer',
  'schedule.command.global_generate.error_footer',
  'recruitment.command.change.panel_quest_unchanged',
  'recruitment.command.change.panel_style_unchanged',
  'recruitment.command.change.panel_date_unchanged',
  'recruitment.command.change.error_prefix',
  'auto_recruitment.operation_error_prefix'
);
```

### 4. Verify old IDs are gone

```sql
SELECT 'master.message_texts' AS source, id, COUNT(*) AS count
FROM master.message_texts
WHERE id IN (
  'schedule.command.generate.success_field_name',
  'schedule.command.global_generate.success_field_name',
  'schedule.command.generate.success_footer',
  'schedule.command.global_generate.success_footer',
  'schedule.command.generate.error_footer',
  'schedule.command.global_generate.error_footer',
  'recruitment.command.change.panel_quest_unchanged',
  'recruitment.command.change.panel_style_unchanged',
  'recruitment.command.change.panel_date_unchanged',
  'recruitment.command.change.error_prefix',
  'auto_recruitment.operation_error_prefix'
)
GROUP BY id
UNION ALL
SELECT 'guild_master.guild_message_texts' AS source, id, COUNT(*) AS count
FROM guild_master.guild_message_texts
WHERE id IN (
  'schedule.command.generate.success_field_name',
  'schedule.command.global_generate.success_field_name',
  'schedule.command.generate.success_footer',
  'schedule.command.global_generate.success_footer',
  'schedule.command.generate.error_footer',
  'schedule.command.global_generate.error_footer',
  'recruitment.command.change.panel_quest_unchanged',
  'recruitment.command.change.panel_style_unchanged',
  'recruitment.command.change.panel_date_unchanged',
  'recruitment.command.change.error_prefix',
  'auto_recruitment.operation_error_prefix'
)
GROUP BY id;
```

If this query returns zero rows, old-ID removal is complete.

## Spreadsheet replacement procedure

Targets:

- Global `messages` sheet (`id`, `message_jp`, `message_en`)
- Guild `guild_messages` sheet (`id`, `message_jp`, `message_en`)

Steps:

1. Replace each `id` according to the ID mapping table above
2. If consolidated IDs duplicate, merge into one row using the same conflict-priority rules
3. Delete old-ID rows
4. Apply the changes:
   - Global: `/gspread_global_load`
   - Guild: `/gspread_load`

Checks:

- No old IDs remain in sheets
- New IDs are not duplicated
- Confirm whether operations are acceptable when `message_en` is empty (English locale falls back to Japanese)
