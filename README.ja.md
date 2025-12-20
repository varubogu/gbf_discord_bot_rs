# GBF Discord Bot (Rust)

[English](README.md) | 日本語

グランブルファンタジー（GBF）のゲーム活動をサポートするDiscord Bot。

## 機能

- 各属性のリアクション（またはボタン）を使ったマルチバトル募集システム（通知、定期募集機能つき）
- 古戦場開始時などのイベント通知システム
- スプレッドシートと連携してデータを注入＆可視化

詳細については下記を参照ください ※日本語のみ
[docs/user](利用者向けマニュアル)

## 必要要件

- Rust 1.70+
- PostgreSQLデータベース
- Discord Botトークン

## セットアップ

1. リポジトリをクローン
2. `.env.example`をコピーして`.env`ファイルを作成し、環境変数を設定する
3. `cargo build --release`を実行
4. `./target/release/gbf_discord_bot_rs`を実行

## コマンド

- `/recruit quest:<クエスト名> [battle_type:<タイプ>] [event_date:<日付>]` - マルチバトル募集を作成

## 主要技術

- **Discord Bot Framework**: poise 0.6.1
- **非同期ランタイム**: tokio 1.47 (multi-thread)
- **ORM**: SeaORM 1.1 (PostgreSQL)
- **エラーハンドリング**: thiserror 1.0
- **ロギング**: tracing 0.1 + tracing-subscriber 0.3
- **テスティング**: tokio-test, mockall

## 開発

### ビルドと実行

```bash
# プロジェクトをビルド
cargo build

# リリースビルド
cargo build --release

# Botを実行（.envの設定が必要）
cargo run

# テストを実行
cargo test

# 特定のテストを実行
cargo test test_name
```

### リントとフォーマット

```bash
# Clippyでコードをチェック
cargo clippy

# コードをフォーマット
cargo fmt

# フォーマットをチェック（変更なし）
cargo fmt -- --check
```

### データベースマイグレーション

```bash
# マイグレーションを実行
cargo run -- migrate

# 新しいマイグレーションを作成
cd migration
sea-orm-cli migrate generate migration_name
```

## ライセンス

このプロジェクトのライセンス情報については、プロジェクトメンテナーにお問い合わせください。
