# Scheduler Processing

## Goals

- Reliably “do something at a fixed time”
- Do not miss tasks that couldn’t run while the bot was down
- Reduce the risk of double execution

## Principles

- Manage tasks in the DB and check the latest DB state right before execution
- Record “executed” after execution to prevent duplicates
- Keep transactions short and do not hold external I/O (Discord sends, etc.) inside

## Notes

- As task types expand (notifications/dissolutions/cleanup), keep separation of responsibilities intact
- During incidents, logs should allow you to trace “which task failed where”
