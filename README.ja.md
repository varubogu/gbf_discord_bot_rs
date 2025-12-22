# GBF Discord Bot (Rust)

[English](README.md) | 日本語

グランブルファンタジー（GBF）のゲーム活動をサポートするDiscord Bot。

## 機能

- 各属性のリアクション（またはボタン）を使ったマルチバトル募集システム
  - クエストによって自動的にリアクション（ボタン）
  - 時間になったら通知する
  - 曜日と時間を決めることで自動的に募集する機能がある
- 古戦場開始時などのイベント通知システム
- スプレッドシートと連携してデータを注入＆可視化

詳細については下記を参照ください ※日本語のみ
[docs/user](利用者向けマニュアル)

## Botコマンド説明

- `/recruit quest:<クエスト名> [battle_type:<タイプ>] [event_date:<日付>]` - マルチバトル募集を作成

※準備中

## 環境構築（開発者向け）

### 必要要件

- Rust 1.70+
- PostgreSQLデータベース
- Discord Botトークン
- Googleスプレッドシート（グローバル用、サーバーごとで計２つ以上）
- Googleスプレッドシート読み書き用の Google Cloud サービスアカウント

※RustとPostgreSQLについてはDocker/DevContainerを使うことで準備が不要になります。

### GoogleCloudサービスアカウントとスプレッドシートのセットアップ

1. Google Cloud > IAMと管理 > サービスアカウント からサービスアカウントを新規作成し、メールアドレスを控える
  - 例: `<アカウント名>@<プロジェクトID>.iam.gserviceaccount.com`
2. サービスアカウントの鍵を作成し、鍵ファイル（json）をダウンロードする
  - 例: `<プロジェクトID>-<サービスアカウントキーID>.json`
2. 下記スプレッドシートをコピーする
  - グローバル: URL準備中
  - サーバーごと: URL準備中
3. コピーしたスプレッドシートに以下の設定を行う
  - 「共有」からGoogle Cloud サービスアカウントのメールアドレスに「編集者」権限を付与
  - その他必要に応じてデータの書き換え

### 開発環境（通常）セットアップ

1. リポジトリをクローン
2. 環境変数のexampleをコピーして環境変数を設定する
  - `.env.app.example` -> `.env.app`
  - `.env.db.example` -> `.env.db`
3. Googleサービスアカウントキーファイルを `プロジェクトフォルダ/.local/` の下へ配置
   例: `<プロジェクトフォルダ>/.local/<プロジェクトID>-<サービスアカウントキーID>.json`
4. PostgreSQLを立ち上げてロール設定を行う
  - ロール設定は `db/sh/init.sh` で行います。
  - `mng.sh`や`mng.ps1`を`dev up`オプションで実行することで
    Dockerコンテナとしてpostgresを動かすことができます。
    例: `./mng.sh prod up`
5. F5キーなどでデバッグ実行

### 開発環境（DevContainer）セットアップ

1. リポジトリをクローン
2. 環境変数のexampleをコピーして環境変数を設定する
  - `.env.app.example` -> `.env.app`
  - `.env.db.example` -> `.env.db`
3. Googleサービスアカウントキーファイルを `プロジェクトフォルダ/.local/` の下へ配置
   例: `<プロジェクトフォルダ>/.local/<プロジェクトID>-<サービスアカウントキーID>.json`
4. F5キーなどでデバッグ実行

※DevContainerの場合、起動したらPostgreSQLも自動的に起動します

### 本番環境（通常）セットアップ

1. リポジトリをクローン
2. 環境変数のexampleをコピーして環境変数を設定する
  - `.env.app.example` -> `.env.app`
  - `.env.db.example` -> `.env.db`
3. Googleサービスアカウントキーファイルを `プロジェクトフォルダ/.local/` の下へ配置
   例: `<プロジェクトフォルダ>/.local/<プロジェクトID>-<サービスアカウントキーID>.json`
4. PostgreSQLを立ち上げてロール設定を行う
  - ロール設定は `db/sh/init.sh` で行います。
  - `mng.sh`や`mng.ps1`を`dev up`オプションで実行することで
    Dockerコンテナとしてpostgresを動かすことができます。
    例: `./mng.sh prod up`
5. `cargo build --release`を実行
6. `./target/release/gbf_discord_bot_rs`を実行

### 本番環境（Docker）セットアップ

1. リポジトリをクローン
2. 環境変数のexampleをコピーして環境変数を設定する
  - `.env.app.example` -> `.env.app`
  - `.env.db.example` -> `.env.db`
3. Googleサービスアカウントキーファイルを `プロジェクトフォルダ/.local/` の下へ配置
   例: `<プロジェクトフォルダ>/.local/<プロジェクトID>-<サービスアカウントキーID>.json`
4. `mng.sh`や`mng.ps1`を`prod up`オプションで実行
   例: `./mng.sh prod up`

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

### 本番環境へのデプロイ補足

このプロジェクトはGitHub Actionsを使用してDockerイメージをビルドし、GitHub Container Registry (GHCR)にプッシュします。本番サーバーではビルド済みイメージをpullするだけなので、リソースを大量に消費するビルド処理を本番サーバー上で実行する必要がありません。

##### 本番環境へのデプロイセットアップ手順

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

## 主要技術

- **Discord Bot Framework**: poise 0.6.1
- **非同期ランタイム**: tokio 1.47 (multi-thread)
- **ORM**: SeaORM 1.1 (PostgreSQL)
- **エラーハンドリング**: thiserror 1.0
- **ロギング**: tracing 0.1 + tracing-subscriber 0.3
- **テスティング**: tokio-test, mockall

## ライセンス

このプロジェクトのライセンス情報については、プロジェクトメンテナーにお問い合わせください。
