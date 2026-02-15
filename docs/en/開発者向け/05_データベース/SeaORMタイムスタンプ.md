# SeaORM Timestamp Conventions

## Purpose

Handle `created_at` / `updated_at` consistently to make auditing and incident investigation easier.

## Principles

- Store timestamps in UTC in the DB
- Ensure `updated_at` always changes on updates (avoid missing updates)

## Notes

- If the app must override timestamps, clarify the reason and confirm the impact scope (search/cleanup)
