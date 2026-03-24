# Integration Test Design (Per Feature)

This folder contains per-feature integration test designs (prerequisite data, case list, expected results, cleanup strategy, etc.).

## Why keep these docs

- Test code alone can make it hard to understand prerequisites, expected results, and cleanup steps
- Real-DB tests can be tedious to recover from when they fail, so design docs help standardize the procedure

## Template (example)

```markdown
# {Feature name} Integration Test Design

## Target use cases

- What to verify (facade name, operation)

## Prerequisites / seed data

- Required guild settings/quests/channels, etc.

## Success cases

- 1-1: Input → expected result

## Failure cases

- 2-1: Missing prerequisites
- 2-2: Missing permissions

## Cleanup

- Tables/conditions to delete (guild_id, etc.)

## Commands

- cargo test {filter}
```
