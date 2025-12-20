# GBF Discord Bot (Rust)

[English](README.md) | 日本語

グランブルファンタジー（GBF）のゲーム活動をサポートするDiscord Bot。

## 機能

- 各属性のリアクションを使ったマルチバトル募集システム
- クエスト情報とバトル募集を保存するデータベース統合
- スラッシュコマンドサポート
- マルチバトル募集時のスタンプ置き換え機能

## 必要要件

- Rust 1.70+
- PostgreSQLデータベース
- Discord Botトークン

## セットアップ

1. リポジトリをクローン
2. configフォルダに`.env`ファイルを作成し、以下の変数を設定：
   ```
   DISCORD_TOKEN=your_discord_bot_token
   GUILD_ID=your_discord_guild_id
   DB_HOST=localhost
   DB_PORT=5432
   DB_NAME=gbf_bot
   GUILD_DB_USER=guild_user
   GUILD_DB_PASSWORD=your_guild_password
   SYSTEM_DB_USER=system_user
   SYSTEM_DB_PASSWORD=your_system_password
   GLOBAL_DB_USER=global_user
   GLOBAL_DB_PASSWORD=your_global_password
   ADMIN_DB_USER=admin_user
   ADMIN_DB_PASSWORD=your_admin_password
   CONFIG_FOLDER=path_to_config_folder
   ```
3. `cargo build --release`を実行
4. `./target/release/gbf_discord_bot_rs`を実行

## コマンド

- `/recruit quest:<クエスト名> [battle_type:<タイプ>] [event_date:<日付>]` - マルチバトル募集を作成

## アーキテクチャ

このプロジェクトはクリーンアーキテクチャを採用しており、以下の層別責務を持ちます：

```
events (Presentation) → facades (Application) → services (Business Logic) → repository (Data Access)
```

詳細は[CLAUDE.md](CLAUDE.md)を参照してください。

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

## Python版からの移行について

### 主な違い

1. **アーキテクチャ**
   - Python: discord.pyのCogシステムを使用
   - Rust: クリーンアーキテクチャによるモジュール化

2. **データベース連携**
   - Python: SQLAlchemy ORM
   - Rust: SeaORM

3. **コマンドハンドリング**
   - Python: プレフィックスコマンドとスラッシュコマンドの混在
   - Rust: poiseによるスラッシュコマンド専用

4. **エラーハンドリング**
   - Python: try/exceptとエラー伝播の混在
   - Rust: Result型による一貫したエラーハンドリング

5. **並行処理**
   - Python: asyncioによる非同期処理
   - Rust: tokioによる非同期ランタイムとコンパイル時の安全性保証

### 移行の課題

1. **API の違い**: discord.pyとpoiseのAPI設計の違いにより、大幅な調整が必要
2. **型システム**: Rustの厳格な型システムにより、オプション値とエラーケースの明示的な処理が必要
3. **データベース統合**: ORMから SeaORM への移行
4. **非同期プログラミング**: PythonとRustのasync/awaitアプローチの違い

### Rust実装の利点

1. **パフォーマンス**: Rustのゼロコスト抽象化による優れたパフォーマンス
2. **安全性**: Rustの所有権システムによる多くの一般的なバグの防止
3. **並行処理**: コンパイル時の保証によるより安全な並行コード
4. **保守性**: 強力な型システムによるコンパイル時のエラー検出

## 今後の改善予定

1. クエスト名のオートコンプリート実装
2. 元のPython Botの追加コマンドの実装
3. エラーハンドリングとユーザーフィードバックの改善
4. コア機能のテスト追加

## ライセンス

このプロジェクトのライセンス情報については、プロジェクトメンテナーにお問い合わせください。
