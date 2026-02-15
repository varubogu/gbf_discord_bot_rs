# DB Overview

## Purpose

This bot stores settings and state (recruitments/notifications, etc.) in the DB.
DB usage is a common source of bugs, so we standardize rules here.

## Multiple schemas (concept)

We use multiple schemas to separate data by purpose.

- `master`: master/reference data shared across all servers
- `guild_*`: per-server settings/master data
- `worker`: runtime data (recruitments, notifications, tasks, etc.)

## Minimum rules

- Only the Facade layer starts/commits/rolls back transactions
- Repositories focus on DB read/write only and contain no business decisions
- Avoid long-running transactions (do not hold external I/O inside)
