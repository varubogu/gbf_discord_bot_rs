# データベースユーザー設計書

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

### 3.2 マイグレーションユーザー

- **想定識別子**: `migration_user`
- **接続情報**: `MIGRATION_URL`環境変数（例: `postgres://migration_user:***@host:5432/gbf_bot_db`）
- **主な利用者**: `sea-orm-cli`、アプリ起動時の自動マイグレーション、DBスキーマ変更用CIジョブ。
- **権限**:
  - 対象データベースへの`CONNECT`
  - 対象スキーマ（通常`public`）での`USAGE`, `CREATE`
  - 既存・新規テーブルに対する`SELECT`, `INSERT`, `UPDATE`, `DELETE`
  - 既存・新規シーケンスに対する`USAGE`, `SELECT`, `UPDATE`
  - 必要に応じて`COMMENT`, `INDEX`, `ALTER TABLE`, `DROP TABLE`を付与（DDL操作用）
- **運用方針**:
  - DDL実行は必ずレビュー済みのマイグレーションに限定し、手動での直接操作は緊急時のみ。
  - 新規テーブル作成時は`ALTER DEFAULT PRIVILEGES`によりアプリユーザーへのDML権限を継承させる。
  - 接続文字列はCI/CD Secretsまたは`.env`で管理し、アプリケーション設定ファイルには含めない。

### 3.3 アプリユーザー

- **想定識別子**: `gbf_bot_user`（テスト環境では`gbf_bot_test_user`）
- **接続情報**: `DATABASE_URL` / `TEST_DATABASE_URL`
- **主な利用者**: Discord Bot本体、統合テスト。
- **権限**:
  - 対象データベースへの`CONNECT`
  - 運用スキーマへの`USAGE`
  - 全テーブルでの`SELECT`, `INSERT`, `UPDATE`, `DELETE`
  - シーケンスでの`USAGE`, `SELECT`, `UPDATE`
  - DDL権限（`CREATE`, `ALTER`, `DROP`）は付与しない。
- **運用方針**:
  - マイグレーション完了後に必要なDMLのみ実行する。
  - `AppState`経由で共有接続プールを利用し、個別にスーパーユーザー接続を確立しない。
  - テスト環境では専用DBおよびユーザー（`TEST_DATABASE_URL`）を用意し、本番データとの混在を避ける。

## 4. 接続情報と環境変数

| 用途 | 変数名 | 例 | 管理場所 |
| ---- | ------ | -- | -------- |
| マイグレーション | `MIGRATION_URL` | `postgres://migration_user:********@host:5432/gbf_bot_db` | `.env`, CI Secrets |
| アプリ本番 | `DATABASE_URL` | `postgres://gbf_bot_user:********@host:5432/gbf_bot_db` | `.env`, Secrets Manager |
| アプリテスト | `TEST_DATABASE_URL` | `postgres://gbf_bot_test_user:********@host:5432/gbf_bot_test_db` | `.env`, CI Secrets |
| DevContainer初期化 | `DB_USER`, `DB_PASSWORD`, `DB_NAME`, `DB_HOST`, `DB_PORT` | `postgres` 等 | `.env`, devcontainer設定 |

- DevContainerでは`db/init.sql`が起動時に実行され、上記ユーザーと権限設定を自動作成する。
- 本番環境ではTerraformやAnsible等のプロビジョニングコードに同等の初期化処理を記述し、平行運用する。

## 5. 運用フロー

1. **初期構築**: デフォルト管理者ユーザーでサーバーをセットアップし、`init.sql`相当のDDLを実行して`migration_user`・`gbf_bot_user`を作成する。
2. **マイグレーション適用**: `sea-orm-cli migrate`またはアプリ起動時の自動実行が`migration_user`で接続し、DDLを適用する。
3. **アプリ稼働**: Bot本体は`DATABASE_URL`経由で`gbf_bot_user`として接続し、CRUDを実施する。
4. **権限変更**: 新しいスキーマ/テーブルを追加した場合、マイグレーションに`ALTER DEFAULT PRIVILEGES`等を含め、アプリユーザーがDML可能であることを確認する。
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
