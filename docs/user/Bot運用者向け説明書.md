# GBF Discord Bot - Bot運用者向けガイド（管理者用）

このガイドでは、Bot運用者（Botのデプロイ・運用・管理者用サーバー管理の責任者）向けに、グローバル設定の管理方法とデプロイメントを説明します。

## 目次

- [概要](#概要)
- [初期セットアップ](#初期セットアップ)
- [権限について](#権限について)
- [グローバルスプレッドシート管理](#グローバルスプレッドシート管理)
- [環境変数設定](#環境変数設定)
- [データベース管理](#データベース管理)
- [デプロイメント](#デプロイメント)
- [監視とログ](#監視とログ)
- [トラブルシューティング](#トラブルシューティング)

---

## 概要

Bot運用者（admin）は、Botのデプロイと運用、全サーバーで共通利用されるグローバルデータを管理します。これには以下が含まれます：

- Botのデプロイメントと監視
- グローバルクエスト情報
- グローバルイベントスケジュール
- マルチ募集種類の定義
- システム全体の環境変数
- データベース管理

### サーバー管理者（guild_master）との違い

| 対象 | 管理範囲 | 権限 |
|-----|---------|------|
| Bot運用者（admin） | Botの運用・デプロイ、全サーバー共通のグローバル設定 | 管理者用サーバーでの実行権限、サーバー管理権限 |
| サーバー管理者（guild_master） | 各Discordサーバー固有の設定 | `gbf_bot_control` ロール |

---

## 初期セットアップ

### 1. 管理サーバーの設定

専用の管理サーバーを作成し、環境変数 `BOT_ADMIN_SERVER_ID` にサーバーIDを設定します。

```bash
BOT_ADMIN_SERVER_ID=<管理サーバーのID>
```

### 2. グローバルスプレッドシートの設定

環境変数 `GLOBAL_SPREADSHEET_ID` にグローバル用スプレッドシートのIDを設定します。

```bash
GLOBAL_SPREADSHEET_ID=<スプレッドシートID>
```

### 3. データベース設定

PostgreSQLデータベースに以下のスキーマが存在することを確認します：

- `master`: グローバルデータ用スキーマ
- `guild_<guild_id>`: 各サーバー用スキーマ

### 4. Botの起動

設定完了後、Botを起動します。

```bash
cargo run
```

---

## 権限について

### 管理サーバー専用コマンド

以下のコマンドは、`BOT_ADMIN_SERVER_ID` で指定された管理サーバーでのみ実行可能です：

- `/グローバルスプレッドシート読み込み`
- `/グローバルスプレッドシート書き込み`

### 権限チェックの仕組み

コマンド実行時、以下をチェックします：

1. コマンドが管理サーバーで実行されているか
2. 環境変数 `BOT_ADMIN_SERVER_ID` が設定されているか

権限がない場合、エラーメッセージが表示されます。

---

## グローバルスプレッドシート管理

グローバルスプレッドシートは、全サーバーで共通利用されるデータを管理します。

### グローバルスプレッドシートの読み込み

#### `/グローバルスプレッドシート読み込み`

グローバルスプレッドシートからデータを読み込み、`master` スキーマに保存します。

**使用例:**

```
/グローバルスプレッドシート読み込み
```

**読み込まれるデータ:**

- `クエスト`: 全サーバーで利用可能なクエスト情報
- `クエスト別名`: クエストの別名定義
- `マルチ募集種類`: 募集タイプの定義
- `(global)イベントスケジュール`: 全サーバー共通のイベント
- `(global)イベント期間内詳細スケジュール`: グローバルイベントの詳細

**注意事項:**

- 読み込みには時間がかかる場合があります
- 既存のグローバルデータは上書きされます
- 読み込み完了後、成功・失敗した行数が表示されます

### グローバルスプレッドシートへの書き込み

#### `/グローバルスプレッドシート書き込み`

`master` スキーマのデータをグローバルスプレッドシートに書き出します。

**使用例:**

```
/グローバルスプレッドシート書き込み
```

**書き込まれるデータ:**

- 統計情報
- システムログ

### グローバルスプレッドシートの構造

グローバルスプレッドシートには以下のシートが必要です：

#### 必須シート

| シート名 | 説明 |
|---------|------|
| クエスト | クエストの基本情報（名前、デフォルト攻略方法など） |
| クエスト別名 | クエストの別名定義 |
| マルチ募集種類 | 募集タイプの定義（DEFAULT, ALL_ELEMENT など） |
| (global)イベントスケジュール | 全サーバー共通のイベント |
| (global)イベント期間内詳細スケジュール | グローバルイベントの詳細 |

#### クエストシートの例

| quest_name | default_battle_type | max_participants | is_enabled |
|-----------|-------------------|------------------|-----------|
| 進撃せし究極の竜HL | 0 | 6 | true |
| 進撃せし蒼き究極の竜HL | 5 | 6 | true |
| 黒銀の翼HL | 1 | 6 | true |

#### クエスト別名シートの例

| quest_name | alias |
|-----------|-------|
| 進撃せし究極の竜HL | アルバハHL |
| 進撃せし究極の竜HL | アルバハ |
| 進撃せし蒼き究極の竜HL | スパバハ |
| 進撃せし蒼き究極の竜HL | スパバハHL |

#### マルチ募集種類シートの例

| id | name | description | reaction_pattern |
|----|------|-------------|-----------------|
| 0 | DEFAULT | 通常の攻略方法 | ✋ |
| 1 | ALL_ELEMENT | 6属性の攻略方法 | 🔴🔵🟤🟢🟡🟣⚪ |
| 2 | SYSTEM | システム狩り | ✋ |
| 3 | RELIC_BUSTER | レリックバスター | ✋ |
| 5 | SUPER_ULTIMATE_BAHAMUT | スパバハ用 | 🔴🔵🟤🟢🟡🟣⚪🔟 |

---

## 環境変数設定

### 必須環境変数

| 変数名 | 説明 | 例 |
|-------|------|-----|
| DATABASE_URL | PostgreSQLデータベースURL | postgresql://user:pass@localhost/gbf_bot |
| DISCORD_TOKEN | DiscordボットトークンDiscord Developer Portalで取得） | MTIzNDU2Nzg5MDEyMzQ1Njc4OQ.ABCDEF... |
| GLOBAL_SPREADSHEET_ID | グローバルスプレッドシートID | ABC123XYZ789... |
| BOT_ADMIN_SERVER_ID | 管理サーバーのID | 123456789012345678 |

### オプション環境変数

| 変数名 | 説明 | デフォルト |
|-------|------|-----------|
| RUST_LOG | ログレベル | info |
| DEFAULT_TIMEZONE | デフォルトタイムゾーン | Asia/Tokyo |

### 環境変数の設定方法

#### .envファイルを使用する場合

プロジェクトルートに `.env` ファイルを作成します：

```bash
DATABASE_URL=postgresql://user:pass@localhost/gbf_bot
DISCORD_TOKEN=MTIzNDU2Nzg5MDEyMzQ1Njc4OQ.ABCDEF...
GLOBAL_SPREADSHEET_ID=ABC123XYZ789...
BOT_ADMIN_SERVER_ID=123456789012345678
RUST_LOG=info
```

#### システム環境変数として設定する場合

```bash
export DATABASE_URL=postgresql://user:pass@localhost/gbf_bot
export DISCORD_TOKEN=MTIzNDU2Nzg5MDEyMzQ1Njc4OQ.ABCDEF...
export GLOBAL_SPREADSHEET_ID=ABC123XYZ789...
export BOT_ADMIN_SERVER_ID=123456789012345678
```

---

## データベース管理

### データベース構造

GBF Discord Botは、PostgreSQLでマルチスキーマ構成を採用しています。

#### スキーマ構成

```
gbf_bot (データベース)
├── master (グローバルデータ)
│   ├── quests
│   ├── quest_aliases
│   ├── battle_recruitment_types
│   └── global_events
└── guild_<guild_id> (各サーバーのデータ)
    ├── guild_settings
    ├── channels
    ├── recruitments
    ├── scheduled_task_recurring_recruitments
    └── guild_events
```

### マイグレーション

データベーススキーマの変更は、SeaORMのマイグレーション機能を使用します。

#### マイグレーションの実行

```bash
cargo run -- migrate
```

#### 新しいマイグレーションの作成

```bash
cd migration
sea-orm-cli migrate generate migration_name
```

### データベースバックアップ

定期的にデータベースのバックアップを取ることを推奨します。

```bash
pg_dump -h localhost -U user -d gbf_bot > backup_$(date +%Y%m%d).sql
```

### データベース復元

```bash
psql -h localhost -U user -d gbf_bot < backup_20260101.sql
```

---

## デプロイメント

### 本番環境へのデプロイ

#### 1. ビルド

```bash
cargo build --release
```

> **注意**: CLAUDE.mdには「DO NOT run `cargo build --release`」とありますが、これは開発中の指示です。本番環境では必ずリリースビルドを使用してください。

#### 2. 実行

```bash
./target/release/gbf_discord_bot_rs
```

### systemdサービスとして実行する場合

`/etc/systemd/system/gbf-bot.service` を作成：

```ini
[Unit]
Description=GBF Discord Bot
After=network.target postgresql.service

[Service]
Type=simple
User=gbf-bot
WorkingDirectory=/opt/gbf-bot
Environment="DATABASE_URL=postgresql://user:pass@localhost/gbf_bot"
Environment="DISCORD_TOKEN=YOUR_TOKEN"
Environment="GLOBAL_SPREADSHEET_ID=YOUR_SPREADSHEET_ID"
Environment="BOT_ADMIN_SERVER_ID=YOUR_SERVER_ID"
ExecStart=/opt/gbf-bot/target/release/gbf_discord_bot_rs
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

サービスの起動：

```bash
sudo systemctl daemon-reload
sudo systemctl enable gbf-bot
sudo systemctl start gbf-bot
```

### Dockerを使用する場合

Dockerfileの例：

```dockerfile
FROM rust:1.47 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/gbf_discord_bot_rs /usr/local/bin/
CMD ["gbf_discord_bot_rs"]
```

docker-compose.ymlの例：

```yaml
version: '3.8'
services:
  bot:
    build: .
    environment:
      - DATABASE_URL=postgresql://user:pass@db/gbf_bot
      - DISCORD_TOKEN=${DISCORD_TOKEN}
      - GLOBAL_SPREADSHEET_ID=${GLOBAL_SPREADSHEET_ID}
      - BOT_ADMIN_SERVER_ID=${BOT_ADMIN_SERVER_ID}
    depends_on:
      - db
    restart: always

  db:
    image: postgres:16
    environment:
      - POSTGRES_DB=gbf_bot
      - POSTGRES_USER=user
      - POSTGRES_PASSWORD=pass
    volumes:
      - pgdata:/var/lib/postgresql/data

volumes:
  pgdata:
```

---

## 監視とログ

### ログレベル

環境変数 `RUST_LOG` でログレベルを設定できます：

```bash
RUST_LOG=trace  # 最も詳細
RUST_LOG=debug  # デバッグ情報
RUST_LOG=info   # 通常の情報（推奨）
RUST_LOG=warn   # 警告のみ
RUST_LOG=error  # エラーのみ
```

### ログの確認

#### 標準出力の場合

```bash
cargo run 2>&1 | tee bot.log
```

#### systemdサービスの場合

```bash
sudo journalctl -u gbf-bot -f
```

### 監視すべき項目

- **CPU使用率**: 異常に高い場合、無限ループなどの可能性
- **メモリ使用率**: メモリリークの検出
- **データベース接続数**: 接続プールの状況
- **エラーログ**: 定期的にエラーを確認

### アラート設定

重要なエラーが発生した場合、通知を受け取れるように設定することを推奨します。

---

## トラブルシューティング

### Botが起動しない

**原因:**

- 環境変数が設定されていない
- データベースに接続できない
- Discordトークンが無効

**対処法:**

1. 環境変数を確認: `printenv | grep -E "DATABASE_URL|DISCORD_TOKEN|GLOBAL_SPREADSHEET_ID|BOT_ADMIN_SERVER_ID"`
2. データベース接続を確認: `psql $DATABASE_URL`
3. Discordトークンを再発行

### グローバルスプレッドシートの読み込みに失敗する

**原因:**

- スプレッドシートIDが間違っている
- スプレッドシートへのアクセス権限がない
- シート名が正しくない

**対処法:**

1. 環境変数 `GLOBAL_SPREADSHEET_ID` を確認
2. スプレッドシートの共有設定を確認（「リンクを知っている全員が閲覧可」）
3. シート名が仕様通りか確認

### データベースマイグレーションに失敗する

**原因:**

- データベース接続権限が不足している
- スキーマが既に存在する

**対処法:**

1. データベースユーザーの権限を確認
2. 既存のスキーマとの競合を確認
3. マイグレーション履歴を確認: `SELECT * FROM seaql_migrations;`

### メモリ使用量が増加し続ける

**原因:**

- メモリリーク
- キャッシュの肥大化

**対処法:**

1. ログレベルを `debug` に設定して詳細を確認
2. Botを再起動
3. 問題が継続する場合、開発者に報告

### 特定のサーバーでコマンドが実行できない

**原因:**

- サーバーのスプレッドシートが未登録
- チャンネル設定が不足
- ロールが正しく設定されていない

**対処法:**

1. サーバー管理者に `/スプレッドシート登録` を実行してもらう
2. サーバー管理者に `/channel_show` でチャンネル設定を確認してもらう
3. `gbf_bot_control` ロールが存在するか確認

---

## セキュリティ

### Discordトークンの管理

- トークンは絶対に公開しない
- `.env` ファイルは `.gitignore` に追加
- トークンが漏洩した場合、直ちに再発行

### データベースセキュリティ

- データベースパスワードは強力なものを使用
- 本番環境では、データベースへの外部アクセスを制限
- 定期的にバックアップを取る

### スプレッドシートセキュリティ

- グローバルスプレッドシートの編集権限は運用者のみに限定
- 共有設定は「リンクを知っている全員が閲覧可」に設定
- 編集履歴を定期的に確認

---

## 開発者向け情報

### プロジェクト構造

```
gbf_discord_bot_rs/
├── src/
│   ├── events/           # イベントハンドラ
│   ├── facades/          # ファサード層
│   ├── services/         # ビジネスロジック
│   ├── repository/       # データアクセス層
│   └── main.rs
├── migration/            # DBマイグレーション
├── docs/                 # ドキュメント
│   ├── develop/         # 開発者向けドキュメント
│   └── user/            # ユーザー向けドキュメント
└── CLAUDE.md            # Claude Code向けガイド
```

### コーディング規約

詳細は `CLAUDE.md` および `.claude/skills/coding-standards` を参照してください。

- 全てのコメントは日本語で記述
- エラーハンドリングには `thiserror` を使用
- ログには `tracing` を使用
- テストには `mockall` を使用

### テストの実行

```bash
cargo test
cargo test test_name          # 特定のテストのみ
cargo test -- --nocapture     # ログ出力あり
```

### リントとフォーマット

```bash
cargo clippy                  # コードチェック
cargo fmt                     # フォーマット
cargo fmt -- --check          # フォーマットチェック
cargo run --bin schema_lint   # スキーマ整合性チェック
```

---

## サポートとコミュニティ

### バグ報告

バグを発見した場合、以下の情報を含めて報告してください：

- Botのバージョン
- エラーメッセージ（ログ）
- 再現手順
- 期待される動作

### 機能リクエスト

新機能のリクエストは、以下を含めて提案してください：

- 機能の説明
- ユースケース
- 既存機能との関係

---

## まとめ

このガイドでは、GBF Discord Botの運用者向けに、グローバル設定の管理方法、デプロイメント、監視、トラブルシューティングについて説明しました。

運用中に問題が発生した場合は、このガイドのトラブルシューティングセクションを参照してください。解決しない場合は、開発者に連絡してください。
