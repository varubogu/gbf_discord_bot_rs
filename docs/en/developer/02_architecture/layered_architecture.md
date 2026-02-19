# Layered Architecture (Developer)

## Purpose

To reduce breakage when adding/changing features and to make testing easier, we split responsibilities and fix the dependency direction.

## Dependency direction (important)

This project depends in the following one-way direction in principle:

`events → facades → services → repository (port)`

Infrastructure implementations are wired at the composition root and are not direct dependencies of services.

## Responsibilities per layer

- **events (presentation layer)**
  - Receive Discord input
  - Validate presentation-level parameters
  - Call facades
- **facades (application layer)**
  - Compose services per use case
  - Own transaction boundaries (begin / commit / rollback)
- **services (domain/business logic)**
  - Implement business rules
  - Depend only on repository traits
- **repository (port layer)**
  - Define persistence contracts (traits)
  - Must not contain ORM-specific behavior
- **infrastructure (adapter layer)**
  - Implement repository traits (e.g. SeaORM adapters)
  - Handle DB-specific behavior and optimizations

## Transactions (important)

- **Only the Facade layer starts/commits/rolls back transactions**
- Services pass the received transaction down to repositories

## Prohibited patterns (examples)

- Calling services/repositories directly from events
- Calling repositories directly from facades (go through services instead)
- Putting business logic in repository adapters
- Making services depend on `SeaOrm*Repository` concrete types

## Related

- [Development rules (entry)](../03_development_rules/README.md)
- [Anti-patterns](../03_development_rules/anti_patterns.md)
- [Dependency injection](dependency_injection.md)
