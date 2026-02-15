# Security

## Principles

- Validate inputs in the presentation layer (events)
- Check both Discord permissions and application-level permissions
- Prefer SeaORM query builder for DB operations; avoid unsafe string concatenation

## Notes for Discord output

- Mentions and links may spread unintentionally; suppress them when needed
- If embedding user input, format it to avoid confusing or misleading output
