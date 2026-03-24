# Unit Tests

## Scope

- Logic that can be completed locally within a single function, service, or transformation
- Examples: input validation, date/time parsing, message generation, and conditional branching

## How to write

- Use AAA: Arrange, Act, Assert
- Cover both success and failure paths at minimum, including boundary values and error cases

## External I/O

- Mock external I/O such as Discord, the DB, and spreadsheets
- Abstract dependencies behind traits and replace them with `mockall`

## Location

- In principle, place tests in `#[cfg(test)] mod tests { ... }` within the target implementation file
