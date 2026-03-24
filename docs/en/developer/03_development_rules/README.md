# Development Rules (Must Read)

This section is where you “return to the rules when you’re unsure”.

## Top priorities (must follow)

- Comments/docs/error messages are written in Japanese (code identifiers are in English)
- Do not break the dependency direction: `events → facades → services → repository`
- Only facades define transaction boundaries
- Do not use `unwrap()` in production code

## Index

- [Workflow](workflow.md)
- [Coding standards](coding_standards.md)
- [Error handling](error_handling.md)
- [Logging](logging.md)
- [Security](security.md)
- [Performance](performance.md)
- [Time zones](time_zones.md)
- [Anti-patterns](anti_patterns.md)
