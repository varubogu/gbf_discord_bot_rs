# Developer Documentation

This folder contains materials for contributors who develop and maintain the bot.
The project assumes **Rust + poise + SeaORM + PostgreSQL**.

## Start here (if you’re unsure)

1. [環境構築](01_introduction/environment_setup.md)
2. [アーキテクチャ概要](02_architecture/layered_architecture.md)
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

- [はじめに](01_introduction/README.md)
- [アーキテクチャ](02_architecture/README.md)
- [開発ルール](03_development_rules/README.md)
- [テスト](04_testing/README.md)
- [データベース](05_database/README.md)
- [機能仕様](06_feature_specifications/README.md)
- [設計メモ（詳細）](07_design_notes/README.md)
