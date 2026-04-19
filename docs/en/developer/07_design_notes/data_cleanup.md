# Data Cleanup

## Purpose

Delete old data (recruitments/notifications/tasks, etc.) to prevent DB growth and search performance degradation.

## Principles

- Run via a maintenance path separated from the main bot
- Fix “when/what to delete” so operators can understand it
- Minimize permissions (DB roles) to avoid accidental deletes

## Operational notes

- In Docker Compose operations, run daily cleanup via `maintenance_scheduler` (built-in scheduler mode)
- Keep `maintenance` for one-shot/manual cleanup
- Deleted data cannot be restored; define a backup policy separately
