# Layered Architecture (Developer)

## Purpose

To reduce breakage when adding/changing features and to make testing easier, we split responsibilities and fix the dependency direction.

## Dependency direction (important)

This project depends in the following one-way direction in principle:

`events → facades → services → repository`

### Responsibilities per layer

- **events (presentation layer)**: Receive Discord input, validate it, and call facades
- **facades (application layer)**: Compose services per use case and manage transaction boundaries
- **services (domain/business logic)**: Implement business rules and use repositories for I/O
- **repository (persistence)**: Read/write the DB only (no business decisions)

## Transactions (important)

- **Only the Facade layer starts/commits/rolls back transactions**
- Services pass the received transaction down to repositories

## Prohibited patterns (examples)

- Calling services/repositories directly from events
- Calling repositories directly from facades (go through services instead)
- Putting business logic in repositories

## Related

- [Development rules (entry)](../03_development_rules/README.md)
- [Anti-patterns](../03_development_rules/anti_patterns.md)
- [Dependency injection](dependency_injection.md)
