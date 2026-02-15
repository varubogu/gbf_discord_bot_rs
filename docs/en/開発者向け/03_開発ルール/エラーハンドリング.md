# Error Handling

## Principles

- Define error types per layer (use `thiserror`)
- Use `#[from]` conversions so callers can handle errors ergonomically

## Returning errors to users

- User-facing messages should clearly suggest the “next action” (e.g., missing permission, missing setup, input format)
- Put internal details in logs; don’t overexpose them to users
