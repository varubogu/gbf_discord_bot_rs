# Coding Conventions

## Minimum rules

- Comments/docs/error messages are written in Japanese
- `unwrap()` is prohibited in production code (consider `panic!()` only when unrecoverable)
- Naming: types in `PascalCase`, functions/vars in `snake_case`, consts in `SCREAMING_SNAKE_CASE`
- Dynamic dispatch (`dyn Trait`) is prohibited; use static dispatch (generics/type parameters)

## Practices that make life easier

- Keep functions short and single-responsibility (readability directly impacts maintenance cost)
- Don’t add `clone()` casually (if you need it, be able to explain why)
- Use `Arc<T>` only where sharing is actually needed
