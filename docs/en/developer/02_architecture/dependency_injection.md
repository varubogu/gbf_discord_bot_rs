# Dependency Injection (DI)

## Purpose

Pass dependencies from the outside to preserve layer boundaries and make the system more change-tolerant and testable.

## Principles

- Aggregate dependencies in `AppState` and pass only what is needed where it is needed
- DB connections are managed by `AppState`; layers must not create ad-hoc new connections
- Don’t use global variables “for convenience” (it hides dependencies and causes accidents)

## How dependencies are passed (practical rules)

### events (presentation layer)

- Validate input and build parameters to pass to facades
- Pull only the minimum required dependencies from `AppState` to construct facades

### facades (transaction boundary)

- Compose services to implement a use case
- Start a transaction and pass it into services

### services / repository

- Services persist through repositories
- Repositories focus on DB read/write only and contain no business decisions

## Substitution for testing

- In unit tests, substitute external I/O (Discord/DB) with test doubles
- Prefer Trait + `mockall`-based substitution (keep it minimal)
