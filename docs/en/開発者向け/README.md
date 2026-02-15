# Developer Documentation

This folder contains materials for contributors who develop and maintain the bot.
The project assumes **Rust + poise + SeaORM + PostgreSQL**.

## Start here (if you’re unsure)

1. [環境構築](01_はじめに/環境構築.md)
2. [アーキテクチャ概要](02_アーキテクチャ/レイヤード構成.md)
3. [Development rules (must-read)](03_開発ルール/README.md)
4. [Testing approach](04_テスト/README.md)

## Common commands (development)

```bash
cargo build -j 1
cargo test -j 1
cargo clippy -j 1
cargo fmt
```

> Note: This repository does not use release builds such as `cargo build --release` (operations assume container/CI artifacts).

## Table of contents

- [はじめに](01_はじめに/README.md)
- [アーキテクチャ](02_アーキテクチャ/README.md)
- [開発ルール](03_開発ルール/README.md)
- [テスト](04_テスト/README.md)
- [データベース](05_データベース/README.md)
- [機能仕様](06_機能仕様/README.md)
- [設計メモ（詳細）](07_設計メモ/README.md)
