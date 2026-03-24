# Developer Documentation

This folder contains materials for contributors who develop and maintain the bot.
The project assumes **Rust + poise + SeaORM + PostgreSQL**.

## Start here (if you’re unsure)

1. [Environment Setup](01_introduction/environment_setup.md)
2. [Architecture Overview](02_architecture/layered_architecture.md)
3. [Development rules (must-read)](03_development_rules/README.md)
4. [Testing approach](04_testing/README.md)

## Common commands (development)

```bash
cargo build -j 1
cargo test -j 1
cargo clippy -j 1
cargo fmt
```

> Note: This repository does not use release builds such as `cargo build --release` (operations assume container/CI artifacts).

## Table of contents

- [Introduction](01_introduction/README.md)
- [Architecture](02_architecture/README.md)
- [Development Rules](03_development_rules/README.md)
- [Testing](04_testing/README.md)
- [Database](05_database/README.md)
- [Feature Specifications](06_feature_specifications/README.md)
- [Design Notes](07_design_notes/README.md)
