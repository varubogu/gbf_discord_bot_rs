# Dependency Injection (DI)

## Purpose

Pass dependencies from the outside to preserve layer boundaries and make the system more change-tolerant and testable.

## Principles

- Aggregate dependencies in `AppState` and pass only what is needed where it is needed
- DB connections are managed by `AppState`; layers must not create ad-hoc new connections
- Keep services dependent on repository traits, not concrete adapters
- Don’t use global variables “for convenience” (it hides dependencies and causes accidents)

## Composition root rule

- Concrete DB adapters (e.g. `SeaOrm*Repository`) are handled only at the composition root.
- The primary wiring point is `src/di/repositories.rs`.
- Outside DI, use repository trait interfaces from `src/repository/**`.

## How dependencies are passed (practical rules)

### events (presentation layer)

- Validate input and build parameters to pass to facades
- Pull only the minimum required dependencies from `AppState` to construct facades

### facades (transaction boundary)

- Compose services to implement a use case
- Start a transaction and pass it into services

### services / repository

- Services persist through repository traits
- Repository ports define contracts only; persistence details stay in infrastructure adapters

## Substitution for testing

- In unit tests, substitute external I/O (Discord/DB) with test doubles
- Prefer Trait + `mockall`-based substitution (keep it minimal)

## Temporary compatibility note

During repository migration, temporary compatibility re-exports may exist under `src/repository/database/**`.
New dependency wiring must use adapter paths under `src/infrastructure/database/repositories/**`.
