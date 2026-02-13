# 開発者向けドキュメント

このフォルダは、開発・保守を行う人向けの資料です。
本プロジェクトは **Rust + poise + SeaORM + PostgreSQL** を前提にしています。

## 最初に読む（迷ったらここ）

1. 環境構築: `docs2/ja/開発者向け/01_はじめに/環境構築.md`
2. アーキテクチャ概要: `docs2/ja/開発者向け/02_アーキテクチャ/レイヤード構成.md`
3. 開発ルール（必読）: `docs2/ja/開発者向け/03_開発ルール/README.md`
4. テストの考え方: `docs2/ja/開発者向け/04_テスト/README.md`

## よく使うコマンド（開発時）

```bash
cargo build
cargo test
cargo clippy
cargo fmt
```

> 注意: このリポジトリでは `cargo build --release` 等のリリースビルドを行いません（運用はコンテナ/CI成果物を前提にします）。

## 目次

- はじめに: `docs2/ja/開発者向け/01_はじめに/README.md`
- アーキテクチャ: `docs2/ja/開発者向け/02_アーキテクチャ/README.md`
- 開発ルール: `docs2/ja/開発者向け/03_開発ルール/README.md`
- テスト: `docs2/ja/開発者向け/04_テスト/README.md`
- データベース: `docs2/ja/開発者向け/05_データベース/README.md`
- 機能仕様: `docs2/ja/開発者向け/06_機能仕様/README.md`
- 設計メモ（詳細）: `docs2/ja/開発者向け/07_設計メモ/README.md`

