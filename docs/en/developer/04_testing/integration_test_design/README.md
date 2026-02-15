# Integration Test Design (Per Feature)

This folder contains per-feature integration test designs (prerequisite data, case list, expected results, cleanup strategy, etc.).

## Why keep these docs

- テストコードだけだと「前提」「期待結果」「片付け方」が読み取りづらくなる
- 実DBテストは失敗時の復旧が面倒になりやすいので、設計書で手順を固定する

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
