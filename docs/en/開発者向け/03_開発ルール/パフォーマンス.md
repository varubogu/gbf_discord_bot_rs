# Performance

## Principles

- Avoid unnecessary `clone()` and prefer borrowing
- Avoid long-running transactions
- Consider concurrency where it helps (e.g., `try_join_all`)
