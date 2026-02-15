# Workflow

## Purpose

Fix the change flow so quality does not degrade with each change.

## Minimal flow (recommended order)

1. Update spec/design (update docs if needed)
2. Implement
3. `cargo clippy` (target only changed files)
4. `cargo fmt` (target only changed files)
5. Test (run focused tests first, then `cargo test` for the full suite)
