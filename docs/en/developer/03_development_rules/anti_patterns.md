# Anti-patterns

Knowing “what not to do” upfront helps prevent accidents.

## Typical examples

- Layer violations (e.g., calling repository directly from events)
- Starting/committing transactions outside facades
- Putting business logic in repositories
- Swallowing exceptions (no logs, and users don’t know what happened)
- Passing user input into DB/Discord without validation
