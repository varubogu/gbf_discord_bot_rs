# データベースユーザー設計書

> **✅ 実装状況: 実装済み**
>
> このドキュメントに記載されているDBロール設計は実装されています。
>
> **実装の確認:**
> - 環境変数: `.env`, `.env.app`, `.env.db` に各ロールのユーザー名・パスワードを設定
> - 実装ファイル: `src/types/app_state.rs` で3つのDB接続を管理
> - ロール構成:
>   - `GUILD_DB_USER` (guild_user): ギルド固有データアクセス、RLS適用
>   - `SYSTEM_DB_USER` (system_user): スケジューラー用、RLS適用なし
>   - `GLOBAL_DB_USER` (global_user): マスターデータ更新用
>   - `ADMIN_DB_USER` (admin_user): マイグレーション用

## 1. 概要

本設計書では、Granblue Fantasy向けDiscord Botが利用するPostgreSQLデータベースにおけるユーザーアカウントの分類・権限・運用指針を定義する。環境ごとの接続手段と秘密情報の扱いを統一し、最小権限に基づく安全な運用を実現する。

### 1.1 作成日時

2025-11-04

### 1.2 対象範囲

- `.devcontainer`配下で構築される開発用PostgreSQLコンテナ
- 本番・ステージング等の永続環境で運用するPostgreSQL
- SeaORMマイグレーションおよびアプリケーション本体が利用する接続

## 2. 共通管理方針

- **職務分離**: スキーマ操作・通常CRUD・運用管理を異なるユーザーで分担する。
- **最小権限**: 付与する権限は役割に必要なDDL/DMLに限定し、`SUPERUSER`および`CREATEDB`はデフォルト管理者のみに許可する。
- **秘密情報管理**: パスワードおよび接続情報は環境変数（`.env`、CIシークレット、Secrets Manager等）で管理し、リポジトリに平文で残さない。
- **監査性**: マイグレーションとアプリの操作ログを分離できるよう、操作主体ごとにユーザーを切り替える。
- **パスワードローテーション**: 四半期ごと、または漏洩兆候検知時に再発行し、関連する環境変数を同時更新する。

## 3. ユーザー別仕様

### 3.1 デフォルト管理者ユーザー

- **想定識別子**: `postgres`（デフォルトスーパーセット）、インフラ側で任意変更可。
- **主な利用者**: 運用管理者、DevContainer初期化処理。
- **権限**:
  - `SUPERUSER`, `CREATEDB`, `CREATEROLE`, `REPLICATION`を保持。
  - 任意スキーマ・ユーザーの作成、緊急時のバックアップ/リストアに利用。
- **利用シナリオ**:
  - DevContainer起動時に`init.sql`を適用し、下位ユーザーを作成。
  - 本番での手動メンテナンス（障害復旧、権限調整、バックアップ）。
- **管理ルール**:
  - アプリケーションやマイグレーションツールからの常用接続は禁止。
  - 認証情報はインフラ管理者のみで共有し、`.env`等には配置しない（DevContainerでは`DB_USER/DB_PASSWORD`を一時利用）。
  - SSHトンネルやBastion経由での限定アクセスを推奨。

### 3.2 管理ユーザー（Admin Role）

- **想定識別子**: `admin_user`
- **接続情報**:
  - `ADMIN_DB_USER` - 管理ユーザー名
  - `ADMIN_DB_PASSWORD` - 管理ユーザーパスワード
  - 共通接続情報（`DB_HOST`, `DB_PORT`, `DB_NAME`）と組み合わせて動的にURL構築
- **主な利用者**: `sea-orm-cli`、アプリ起動時の自動マイグレーション、DBスキーマ変更用CIジョブ。
- **権限**:
  - 対象データベースへの`CONNECT`
  - 対象スキーマでの`USAGE`, `CREATE`
  - 既存・新規テーブルに対する`SELECT`, `INSERT`, `UPDATE`, `DELETE`
  - 既存・新規シーケンスに対する`USAGE`, `SELECT`, `UPDATE`
  - 必要に応じて`COMMENT`, `INDEX`, `ALTER TABLE`, `DROP TABLE`を付与（DDL操作用）
- **運用方針**:
  - DDL実行は必ずレビュー済みのマイグレーションに限定し、手動での直接操作は緊急時のみ。
  - 新規テーブル作成時は`ALTER DEFAULT PRIVILEGES`によりアプリユーザーへのDML権限を継承させる。
  - 認証情報はCI/CD Secretsまたは`.env`で管理し、アプリケーション設定ファイルには含めない。

### 3.3 アプリユーザー（Guild/System/Global Role）

本アプリケーションでは、スキーマ別にアクセス権限を分離した3つのロールを使用します。

#### 3.3.1 Guild Role

- **想定識別子**: `guild_user`
- **接続情報**:
  - `GUILD_DB_USER` - Guildロールユーザー名
  - `GUILD_DB_PASSWORD` - Guildロールパスワード
  - 共通接続情報（`DB_HOST`, `DB_PORT`, `DB_NAME`）と組み合わせて動的にURL構築
- **主な利用者**: Discord Bot本体（サーバー固有データアクセス時）
- **権限**:
  - `guild`スキーマへの`USAGE`および全テーブルでの`SELECT`, `INSERT`, `UPDATE`, `DELETE`
  - DDL権限（`CREATE`, `ALTER`, `DROP`）は付与しない
- **運用方針**:
  - サーバー固有のバトル募集、参加者情報等へのアクセスに使用

#### 3.3.2 System Role

- **想定識別子**: `system_user`
- **接続情報**:
  - `SYSTEM_DB_USER` - Systemロールユーザー名
  - `SYSTEM_DB_PASSWORD` - Systemロールパスワード
- **主な利用者**: Discord Bot本体（システム設定、スケジュール情報アクセス時）
- **権限**:
  - `system`スキーマへの`USAGE`および全テーブルでの`SELECT`, `INSERT`, `UPDATE`, `DELETE`
  - DDL権限は付与しない
- **運用方針**:
  - システム全体の設定、通知スケジュール等へのアクセスに使用

#### 3.3.3 Global Role

- **想定識別子**: `global_user`
- **接続情報**:
  - `GLOBAL_DB_USER` - Globalロールユーザー名
  - `GLOBAL_DB_PASSWORD` - Globalロールパスワード
- **主な利用者**: Discord Bot本体（グローバルマスタデータアクセス時）
- **権限**:
  - `global`スキーマへの`USAGE`および全テーブルでの`SELECT`, `INSERT`, `UPDATE`, `DELETE`
  - DDL権限は付与しない
- **運用方針**:
  - クエスト情報、ボス情報等のマスタデータへのアクセスに使用
  - マスタデータは原則読み取り専用だが、更新機能実装時に備えて書き込み権限も付与

#### 3.3.4 テスト環境

- **接続情報**:
  - `TEST_DB_HOST`, `TEST_DB_PORT`, `TEST_DB_NAME` - テスト用データベース接続情報
  - `TEST_DB_USER`, `TEST_DB_PASSWORD` - テスト用ユーザー認証情報
- **運用方針**:
  - テスト環境では専用DBおよびユーザーを用意し、本番データとの混在を避ける
  - `AppState`経由で共有接続プールを利用し、個別にスーパーユーザー接続を確立しない

## 4. 接続情報と環境変数

### 4.1 共通接続情報

| 変数名 | 説明 | 例 | 管理場所 |
| ------ | ---- | -- | -------- |
| `DB_HOST` | データベースホスト名 | `localhost` | `.env`, CI Secrets |
| `DB_PORT` | データベースポート番号 | `5432` | `.env`, CI Secrets |
| `DB_NAME` | データベース名 | `gbf_bot_db` | `.env`, CI Secrets |

### 4.2 ロール別認証情報

| 用途 | ユーザー名変数 | パスワード変数 | 管理場所 |
| ---- | -------------- | -------------- | -------- |
| Admin（マイグレーション） | `ADMIN_DB_USER` | `ADMIN_DB_PASSWORD` | `.env`, CI Secrets |
| Guild（サーバー固有データ） | `GUILD_DB_USER` | `GUILD_DB_PASSWORD` | `.env`, Secrets Manager |
| System（システム設定） | `SYSTEM_DB_USER` | `SYSTEM_DB_PASSWORD` | `.env`, Secrets Manager |
| Global（マスタデータ） | `GLOBAL_DB_USER` | `GLOBAL_DB_PASSWORD` | `.env`, Secrets Manager |

### 4.3 テスト環境

| 変数名 | 説明 | 例 | 管理場所 |
| ------ | ---- | -- | -------- |
| `TEST_DB_HOST` | テスト用DBホスト名 | `localhost` | `.env`, CI Secrets |
| `TEST_DB_PORT` | テスト用DBポート番号 | `5433` | `.env`, CI Secrets |
| `TEST_DB_NAME` | テスト用データベース名 | `gbf_bot_test_db` | `.env`, CI Secrets |
| `TEST_DB_USER` | テスト用ユーザー名 | `test_user` | `.env`, CI Secrets |
| `TEST_DB_PASSWORD` | テスト用パスワード | `test_password` | `.env`, CI Secrets |

### 4.4 DevContainer初期化

| 変数名 | 説明 | 管理場所 |
| ------ | ---- | -------- |
| `DB_USER`, `DB_PASSWORD` | デフォルト管理者認証情報 | `.env`, devcontainer設定 |

### 4.5 接続URL構築

アプリケーション起動時、上記環境変数から以下の形式で動的にURL構築：

```
postgres://{ROLE_USER}:{ROLE_PASSWORD}@{DB_HOST}:{DB_PORT}/{DB_NAME}
```

例:
- Guild Role: `postgres://guild_user:***@localhost:5432/gbf_bot_db`
- Admin Role: `postgres://admin_user:***@localhost:5432/gbf_bot_db`

- DevContainerでは`db/init.sql`が起動時に実行され、上記ユーザーと権限設定を自動作成する。
- 本番環境ではTerraformやAnsible等のプロビジョニングコードに同等の初期化処理を記述し、平行運用する。

## 5. 運用フロー

1. **初期構築**: デフォルト管理者ユーザーでサーバーをセットアップし、`init.sql`相当のDDLを実行して`admin_user`、`guild_user`、`system_user`、`global_user`を作成する。
2. **マイグレーション適用**: `sea-orm-cli migrate`またはアプリ起動時の自動実行が、`ADMIN_DB_USER`/`ADMIN_DB_PASSWORD`と共通接続情報から構築されたURLで接続し、DDLを適用する。
3. **アプリ稼働**: Bot本体は操作対象スキーマに応じて適切なロール（Guild/System/Global）の認証情報から構築されたURLで接続し、CRUDを実施する。
4. **権限変更**: 新しいスキーマ/テーブルを追加した場合、マイグレーションに`ALTER DEFAULT PRIVILEGES`等を含め、各ロールがDML可能であることを確認する。
5. **監査・ローテーション**: 四半期ごとにパスワード更新、接続テスト、不要ユーザーの削除を実施し、更新履歴を運用記録に残す。

## 6. セキュリティおよび監査留意点

- 認証情報は commit 禁止。Secrets のアクセス権はBot運用チームに限定する。
- PostgreSQLの`log_connections`と`log_disconnections`を有効化し、ユーザーごとのアクセス履歴を確認できるようにする。
- 侵入検知時は以下の手順で対応する：
  1. デフォルト管理者ユーザーで対象ユーザーの`ALTER ROLE ... NOLOGIN`を実施。
  2. 新パスワードを生成し、Secretsを更新。
  3. アプリケーションを再起動して新しい接続情報を反映。
- 本番環境ではIP制限・SSL/TLS接続を必須とし、DevContainerのローカル接続のみ例外とする。

## 7. チェックリスト

- [ ] 本番・開発・テストでユーザーと権限の組み合わせが一致しているか確認した。
- [ ] マイグレーションで新規テーブルを作成する際、アプリユーザーへのDML権限継承を設定した。
- [ ] Secretsと`.env`の内容が最新のパスワードと一致している。
- [ ] 監査ログおよびパスワードローテーションの記録を運用ドキュメントへ反映した。
- [ ] 緊急対応手順（NOLOGIN化、権限剥奪）がRunbookとして整理されている。
