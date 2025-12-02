# データベースロール使用ガイド

## 概要

このプロジェクトでは、PostgreSQLのRow Level Security (RLS)とデータベースロールを使用して、セキュリティとデータ分離を実現しています。

用途に応じて4つのロールを使い分けることで、以下を実現します：
- ギルドごとのデータ分離（RLS）
- マスターデータの保護
- 最小権限の原則に基づいたアクセス制御

## データベースロール一覧

### 1. System ロール (`gbf_bot_system`)

**用途**: スケジューラー、バックグラウンドタスク

**権限**:
- master スキーマ: SELECT only（読み取り専用）
- guild_master スキーマ: CRUD（全操作可能）
- worker スキーマ: CRUD（全操作可能）
- RLS: BYPASSRLS（RLS適用なし）

**使用例**:
- スケジュール通知の処理
- 定期的なクリーンアップタスク
- 全ギルド対象のバッチ処理

**コード例**:
```rust
use crate::types::DbRole;

// スケジューラー処理でSystemロールを使用
let database_url = config.database_url(DbRole::System)?;
let db = Database::connect(database_url).await?;
```

---

### 2. Guild ロール (`gbf_bot_guild`)

**用途**: 通常のDiscordコマンド実行、Bot操作

**権限**:
- master スキーマ: SELECT only（読み取り専用）
- guild_master スキーマ: CRUD（全操作可能）
- worker スキーマ: CRUD（全操作可能）
- RLS: **適用あり**（guild_id制限）

**RLSポリシー**:
```sql
-- guild_master, worker スキーマのテーブルは guild_id でフィルタリングされる
WHERE guild_id = current_setting('app.current_guild_id')::bigint
```

**使用例**:
- `/recruit` コマンド実行
- `/quest` コマンド実行
- 通常のBot操作全般

**コード例**:
```rust
use crate::types::DbRole;

// メインアプリケーションはGuildロールを使用（デフォルト）
let database_url = config.database_url(DbRole::Guild)?;
let db = Database::connect(database_url).await?;

// RLS用のguild_id設定（必須）
db.execute(Statement::from_string(
    DatabaseBackend::Postgres,
    format!("SET app.current_guild_id = {}", guild_id)
)).await?;
```

**注意事項**:
- **必ず** `app.current_guild_id` を設定してから操作すること
- 設定しない場合、RLSポリシーにより全てのデータが見えなくなる

---

### 3. Global ロール (`gbf_bot_global`)

**用途**: マスターデータ更新、スプレッドシート同期

**権限**:
- master スキーマ: CRUD（全操作可能）
- guild_master スキーマ: CRUD（全操作可能）
- worker スキーマ: CRUD（全操作可能）
- RLS: BYPASSRLS（RLS適用なし）

**使用例**:
- グローバルスプレッドシートからのマスターデータ同期
- クエストマスターデータの更新
- バトルスタイル、属性などの更新

**コード例**:
```rust
use crate::types::DbRole;

// スプレッドシート同期処理でGlobalロールを使用
pub async fn sync_global_spreadsheet(config: &AppConfig) -> Result<()> {
    let database_url = config.database_url(DbRole::Global)?;
    let db = Database::connect(database_url).await?;

    // masterスキーマへの書き込みが可能
    quest::Entity::insert(new_quest)
        .exec(&db)
        .await?;

    Ok(())
}
```

---

### 4. Admin ロール (`gbf_bot_admin`)

**用途**: マイグレーション実行、スキーマ変更、管理操作

**権限**:
- 全スキーマ: CRUD（全操作可能）
- スキーマ作成・変更・削除
- RLS: BYPASSRLS（RLS適用なし）

**使用例**:
- データベースマイグレーション
- スキーマ変更
- 緊急時のデータ修復

**コード例**:
```bash
# マイグレーション実行（自動的にAdminロールを使用）
cargo run --bin migration
```

**注意事項**:
- 本番環境での手動使用は最小限に
- マイグレーション以外での使用は記録を残すこと

---

## 環境変数設定

### .env ファイル例

```bash
# データベース接続情報（共通）
DB_HOST=localhost
DB_PORT=5432
DB_NAME=gbf_bot_db

# Systemロール
SYSTEM_DB_USER=gbf_bot_system
SYSTEM_DB_PASSWORD=your_system_password

# Guildロール
GUILD_DB_USER=gbf_bot_guild
GUILD_DB_PASSWORD=your_guild_password

# Globalロール
GLOBAL_DB_USER=gbf_bot_global
GLOBAL_DB_PASSWORD=your_global_password

# Adminロール
ADMIN_DB_USER=gbf_bot_admin
ADMIN_DB_PASSWORD=your_admin_password
```

---

## コードでの使用方法

### DbRole enumの使用

```rust
use crate::types::{AppConfig, DbRole};

// 設定ファイルから読み込み
let config = AppConfig::from_env()?;

// 用途に応じてロールを選択
match use_case {
    UseCase::DiscordCommand => {
        let url = config.database_url(DbRole::Guild)?;
        // RLS適用、guild_id制限あり
    }
    UseCase::Scheduler => {
        let url = config.database_url(DbRole::System)?;
        // RLS適用なし、全ギルドアクセス可能
    }
    UseCase::SpreadsheetSync => {
        let url = config.database_url(DbRole::Global)?;
        // masterスキーマへの書き込み可能
    }
    UseCase::Migration => {
        let url = config.database_url(DbRole::Admin)?;
        // 全権限
    }
}
```

### AppStateでの実装例

将来的にAppStateに複数のDB接続を保持する場合:

```rust
pub struct AppState {
    pub guild_db: DatabaseConnection,   // DbRole::Guild
    pub system_db: DatabaseConnection,  // DbRole::System
    pub global_db: DatabaseConnection,  // DbRole::Global
    pub config: AppConfig,
}

impl AppState {
    pub async fn new(config: AppConfig) -> Result<Self> {
        let guild_db = Database::connect(
            config.database_url(DbRole::Guild)?
        ).await?;

        let system_db = Database::connect(
            config.database_url(DbRole::System)?
        ).await?;

        let global_db = Database::connect(
            config.database_url(DbRole::Global)?
        ).await?;

        Ok(Self {
            guild_db,
            system_db,
            global_db,
            config,
        })
    }
}
```

---

## セットアップ手順

### 1. ロール作成

```bash
# .envファイルを準備
cp .env.example .env
# パスワードを設定

# ロール作成スクリプトを実行
bash scripts/setup_db_roles.sh --env-file .env
```

### 2. マイグレーション実行

```bash
# 環境変数を読み込み
source .env

# マイグレーション実行（自動的にAdminロールを使用）
cargo run --bin migration
```

### 3. アプリケーション実行

```bash
# 通常実行（Guildロールを使用）
cargo run
```

---

## トラブルシューティング

### RLSでデータが見えない

**症状**: クエリ実行時にデータが0件返される

**原因**: `app.current_guild_id` が設定されていない

**解決方法**:
```rust
// 各リクエストの最初に設定
db.execute(Statement::from_string(
    DatabaseBackend::Postgres,
    format!("SET app.current_guild_id = {}", guild_id)
)).await?;
```

### 権限エラー

**症状**: `permission denied for schema master`

**原因**: 不適切なロールを使用している

**解決方法**:
- masterスキーマへの書き込み → `DbRole::Global` または `DbRole::Admin`
- RLSバイパスが必要 → `DbRole::System`, `DbRole::Global`, または `DbRole::Admin`

### パスワードエラー

**症状**: `password authentication failed`

**原因**: 環境変数が設定されていない、または誤っている

**解決方法**:
```bash
# 環境変数を確認
echo $GUILD_DB_USER
echo $GUILD_DB_PASSWORD

# .envファイルを再確認
cat .env | grep DB_
```

---

## ベストプラクティス

1. **最小権限の原則**: 必要最小限のロールを使用する
   - 通常操作は `DbRole::Guild`
   - バックグラウンド処理は `DbRole::System`
   - マスターデータ更新のみ `DbRole::Global`

2. **RLS設定の徹底**: Guildロール使用時は必ず `app.current_guild_id` を設定

3. **ログ記録**: どのロールを使用したか記録する
   ```rust
   info!("Using database role: {}", DbRole::Guild.description());
   ```

4. **テスト**: 各ロールで期待通りの動作をするかテストする

5. **監査**: Adminロールの使用は監査ログに記録する
