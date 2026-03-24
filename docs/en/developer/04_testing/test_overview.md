# Overall Test Design

## Goals

- Quickly confirm that a change did not break existing behavior
- Preserve bugs as reproducible cases so they do not reoccur
- Protect layer responsibilities, especially transaction boundaries

## Test types

### Unit tests

- Fast and easy to localize when they fail
- External I/O such as Discord, the DB, and spreadsheets should generally be mocked

### Integration tests

- Start from facades and verify consistency across multiple layers
- Use test environments that are close to production for external I/O such as Discord, the DB, and spreadsheets

### System/E2E tests

- Excluded from automated coverage for now because Discord interaction automation is difficult under platform constraints

## Common rules

- Write tests in AAA order: Arrange, Act, Assert
- Do not share state between tests or depend on execution order
- Assert business-meaningful outcomes rather than implementation details
