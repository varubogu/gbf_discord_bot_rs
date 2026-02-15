# Connections and Transactions

## Overview

This project separates DB connection management from transaction management,
while preserving the layered architecture (`events → facades → services → repository`).

## Design goals

- Prevent ad-hoc connections and reduce operational failures
- Standardize transaction boundaries and preserve consistency
- Localize dependencies on implementation details (ORM)
- Stay resilient to future architectural changes

## Design principles

### ORM independence

- Transaction abstraction (`DatabaseTransactionTrait`)
- Connection abstraction (`DatabaseConnectionTrait`)
- Do not leak ORM-specific types into upper layers

### Separation of responsibilities by layer

- Facade: use-case execution and transaction boundaries
- Service: business rule implementation
- Repository: persistence

### AppState pattern

- Shared connections are managed by `AppState`
- Do not create ad-hoc connections in each layer

## What must be enforced

- Connections are managed by `AppState`; layers must not create new connections freely
- Transaction boundaries are managed in the Facade layer
- Services only pass the received transaction down to repositories
- Repositories focus on persistence and contain no business decisions

## Key components and responsibilities

### AppState

- Holds DB connections shared across the app
- Entry point for obtaining connections per use case

### TransactionManager (facade-side coordinator)

- Start transactions
- Commit / rollback based on outcome
- Guarantee rollback on failure
- Provide an execution context for repositories

### TransactionContext

- Execution context bundling a transaction and repository access
- Shared container when facades coordinate multiple services

### Service

- Implement business rules
- Do not start/end transactions

### Repository

- Execute queries within a transaction
- Maintain consistency of CRUD operations

## Typical flow

1. The facade starts a transaction
2. The facade coordinates and runs multiple services
3. Services pass the transaction to repositories
4. If everything succeeds, commit
5. If anything fails, rollback

## How to choose transaction boundaries

- Group updates that must be consistent within a use case into the same boundary
- Move external API calls outside the boundary as much as possible
- Split boundaries so DB locks aren’t held for long

## Failure model

- If failure happens mid-update, rollback and do not leave partial updates
- For flows that include external I/O (Discord sends, etc.), define ordering and retry/replay strategy
- If rollback itself fails, capture it as an `error` log to ensure investigability

## Transaction design principles

- Avoid long transactions
- Do not hold external API waits inside transactions
- Keep updates minimal and short
- Even within a single use case, clarify boundaries by responsibility

## Anti-patterns

- Starting transactions directly in the events layer
- Committing/rolling back in the Service layer
- Putting business branching in repositories
- Continuing without rollback on failure

## Best practices

- Fix the operation order inside facades and leave observable logs
- For critical updates, re-check the target before updating
- Do concurrency outside the transaction; keep the update phase short

## Checklist

- Does the facade explicitly own the transaction boundary?
- Are services free from transaction control?
- Do repositories focus on DB operations only?
- Is failure behavior for external I/O (retries/compensation) defined?
- Are logs designed for rollback failures?

## Related documents

- [layered_architecture.md](../02_architecture/layered_architecture.md)
- [error_handling.md](../03_development_rules/error_handling.md)
- [logging.md](../03_development_rules/logging.md)
