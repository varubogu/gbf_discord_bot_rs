# Coding Conventions

## Minimum rules

- Comments/docs/error messages are written in Japanese
- `unwrap()` is prohibited in production code (consider `panic!()` only when unrecoverable)
- Naming: types in `PascalCase`, functions/vars in `snake_case`, consts in `SCREAMING_SNAKE_CASE`
- Dynamic dispatch (`dyn Trait`) is prohibited; use static dispatch (generics/type parameters)
- User-facing strings (message content, embeds, labels/placeholders for buttons/selects/modals, and interaction responses) must be defined in `locales/messages.yml` and retrieved via `MessageTextId` / `MessageService`. Hardcoding user-facing text in Rust code is prohibited.

## Master table ID enums

When a master table's ID column has a **fixed, code-controlled set of values** (i.e., values that are managed in the codebase rather than entered freely by users), define the values as a Rust enum inside the entity file for that table.

Rules:
- Define the enum in `src/models/entities/<schema>/<table>.rs`, alongside the SeaORM entity
- Implement `as_i32(&self) -> i32` and `from_i32(value: i32) -> Option<Self>` helpers
- Always keep the enum values in sync with the actual DB master records
- Do NOT define bare `const` values for these IDs — use the enum variants

Example: `ScheduledTaskType` in `src/models/entities/worker/scheduled_tasks.rs`
Example: `GuildChannelType` in `src/models/entities/master/channel_types.rs`

## Practices that make life easier

- Keep functions short and single-responsibility (readability directly impacts maintenance cost)
- Don’t add `clone()` casually (if you need it, be able to explain why)
- Use `Arc<T>` only where sharing is actually needed
