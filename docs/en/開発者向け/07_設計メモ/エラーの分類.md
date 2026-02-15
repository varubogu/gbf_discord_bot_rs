# Error Classification

## Purpose

Separate “expected failures” from “incidents” in operations, and connect them to appropriate logging/alerting/user messaging.

## Typical categories

- Input errors (users can fix)
- Permission errors (roles/admin server, etc.)
- Missing prerequisites (incomplete setup, master data not synced, etc.)
- External failures (Discord/DB/Spreadsheet)

## Policy

- Keep user-facing messages short and suggest the next action
- Put details in logs so issues can be investigated
