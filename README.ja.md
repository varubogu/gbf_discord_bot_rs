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

## 本番環境へのデプロイ

このプロジェクトはGitHub Actionsを使用してDockerイメージをビルドし、GitHub Container Registry (GHCR)にプッシュします。本番サーバーではビルド済みイメージをpullするだけなので、リソースを大量に消費するビルド処理を本番サーバー上で実行する必要がありません。

### セットアップ手順

1. **GitHub Container Registryの有効化**
   - リポジトリの Settings > Actions > General に移動
   - GITHUB_TOKENの "Read and write permissions" を有効化

2. **本番サーバーの設定**
   ```bash
   # リポジトリをクローン
   git clone https://github.com/your-username/gbf_discord_bot_rs.git
   cd gbf_discord_bot_rs

   # GITHUB_REPOSITORY環境変数を設定
   export GITHUB_REPOSITORY=your-username/gbf_discord_bot_rs

   # GitHub Container Registryにログイン
   echo $GITHUB_TOKEN | docker login ghcr.io -u your-username --password-stdin

   # 環境変数ファイルを作成
   cp .env.app.example .env.app
   cp .env.db.example .env.db
   # .env.appと.env.dbを編集して設定を行う

   # Google Service Account Key用の.localディレクトリを作成
   mkdir -p .local
   # サービスアカウントキーファイルを.local/に配置

   # サービスをpullして起動
   docker-compose pull
   docker-compose up -d
   # または管理スクリプトを使用
   ./mng.sh prod up
   ```

3. **自動デプロイ**
   - `main`ブランチへのpushで自動的にビルド＆GHCRへプッシュ
   - 本番サーバーでpullして再起動:
   ```bash
   docker-compose pull
   docker-compose up -d
   # または管理スクリプトを使用
   ./mng.sh prod up
   ```

### 開発環境 vs 本番環境

- **開発環境**: `./mng.sh dev up`でデータベースをローカルでDockerで実行
- **本番環境** (`docker-compose.yml`): GHCRからビルド済みイメージをpull（ローカルビルド不要）

## ライセンス

このプロジェクトのライセンス情報については、プロジェクトメンテナーにお問い合わせください。
