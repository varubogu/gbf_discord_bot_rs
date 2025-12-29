# データクリーンアップシステム - データベースロール設計

## 概要

データクリーンアップシステムは専用のデータベースロール（`gbf_bot_cleanup`）を使用し、最小権限の原則に基づいてアクセス制御を行います。

## Cleanupロール (`gbf_bot_cleanup`)

### 用途
- データクリーンアップバッチ専用
- 30日以上前の古いデータの削除

### 権限

#### workerスキーマ
- **DELETE権限のみ**を以下のテーブルに付与:
  - `battle_recruitments` - マルチ募集
  - `notifications` - 通知
  - `scheduled_tasks` - スケジュールタスク

#### masterスキーマ
- **権限なし**（マスターデータは読み取りも不要）

#### guild_masterスキーマ
- **権限なし**（ギルド設定は触らない）

#### RLS（Row Level Security）
- **BYPASSRLS**（RLS適用なし）
- 理由: 全ギルドのデータを対象とするため

### SQL定義

```sql
-- Cleanupロールを作成
CREATE ROLE gbf_bot_cleanup WITH LOGIN PASSWORD 'cleanup_password_here';

-- workerスキーマへの接続権限
GRANT USAGE ON SCHEMA worker TO gbf_bot_cleanup;

-- 削除対象テーブルにDELETE権限のみ付与
GRANT DELETE ON worker.battle_recruitments TO gbf_bot_cleanup;
GRANT DELETE ON worker.notifications TO gbf_bot_cleanup;
GRANT DELETE ON worker.scheduled_tasks TO gbf_bot_cleanup;

-- RLSをバイパス（全ギルドのデータを削除するため）
ALTER ROLE gbf_bot_cleanup WITH BYPASSRLS;

-- トランザクション用に必要な最小権限
GRANT USAGE ON SCHEMA worker TO gbf_bot_cleanup;

-- 削除対象テーブルへのSELECT権限も必要（削除条件の確認のため）
GRANT SELECT ON worker.battle_recruitments TO gbf_bot_cleanup;
GRANT SELECT ON worker.notifications TO gbf_bot_cleanup;
GRANT SELECT ON worker.scheduled_tasks TO gbf_bot_cleanup;
```

### セキュリティ上の利点

1. **最小権限**: DELETE権限のみ、特定テーブルのみ
2. **誤操作防止**: masterやguild_masterスキーマへのアクセス不可
3. **監査可能**: cleanup専用ロールなので、削除操作の追跡が容易
4. **分離**: 他のロール（System、Guild、Global）とは完全に独立

### 環境変数設定

#### .env.maintenance
```bash
# Cleanupロールで接続
DB_HOST=db
DB_PORT=5432
DB_USER=gbf_bot_cleanup
DB_PASSWORD=cleanup_password_here
DB_NAME=gbf_bot_db

# クリーンアップ設定
CLEANUP_RETENTION_DAYS=30
RUST_LOG=info
```

## 既存ロールとの比較

| ロール | master | guild_master | worker | RLS | 用途 |
|--------|--------|--------------|--------|-----|------|
| **Guild** | SELECT | CRUD | CRUD | 適用あり | 通常のBot操作 |
| **System** | SELECT | CRUD | CRUD | BYPASSRLS | スケジューラー |
| **Global** | CRUD | CRUD | CRUD | BYPASSRLS | マスターデータ更新 |
| **Cleanup** | なし | なし | DELETE+SELECT (特定テーブル) | BYPASSRLS | データクリーンアップ |

## 削除可能なテーブル一覧

### worker.battle_recruitments
- **削除条件**: `quest_start_at < cleanup_before AND is_recruiting = false`
- **CASCADE削除**:
  - `recruitment_participants`
  - `battle_recruitment_dismissals`
  - `notification_rel_battle_recruitments`
  - `scheduled_task_dissolutions`
  - `scheduled_task_dismissals`

### worker.notifications
- **削除条件**: `schedule_datetime < cleanup_before AND is_sent = true`
- **CASCADE削除**:
  - `notification_rel_battle_recruitments`
  - `notification_rel_event_schedules`
  - `scheduled_task_notifications`

### worker.scheduled_tasks
- **削除条件**: `schedule_datetime < cleanup_before AND is_executed = true AND task_type != 3`
- **CASCADE削除**:
  - `scheduled_task_notifications`
  - `scheduled_task_dissolutions`
  - `scheduled_task_dismissals`
  - `scheduled_task_recurring_recruitments`
  - `scheduled_task_cleanups`

## 削除できないテーブル（権限なし）

### masterスキーマ（全テーブル）
- `quests`
- `battle_styles`
- `elements`
- `event_schedules`
- `event_schedule_details`
- など

### guild_masterスキーマ（全テーブル）
- `battle_recruitment_schedules`
- `battle_recruitment_schedule_days`
- `guild_event_schedules`
- など

### workerスキーマ（権限がないテーブル）
- `recruitment_participants` - CASCADE削除のみ
- `notification_rel_battle_recruitments` - CASCADE削除のみ
- など

## マイグレーション

### 新規ロール作成マイグレーション

`migration/src/mXXXXXXXXXX_create_cleanup_role.rs`:

```rust
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Cleanupロールを作成
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                -- Cleanupロールを作成
                CREATE ROLE gbf_bot_cleanup WITH LOGIN PASSWORD 'cleanup_password_here';

                -- workerスキーマへの接続権限
                GRANT USAGE ON SCHEMA worker TO gbf_bot_cleanup;

                -- 削除対象テーブルにDELETE + SELECT権限を付与
                GRANT DELETE, SELECT ON worker.battle_recruitments TO gbf_bot_cleanup;
                GRANT DELETE, SELECT ON worker.notifications TO gbf_bot_cleanup;
                GRANT DELETE, SELECT ON worker.scheduled_tasks TO gbf_bot_cleanup;

                -- RLSをバイパス
                ALTER ROLE gbf_bot_cleanup WITH BYPASSRLS;
                "#,
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP ROLE IF EXISTS gbf_bot_cleanup;")
            .await?;

        Ok(())
    }
}
```

## テスト

### 権限テスト

```sql
-- Cleanupロールで接続
SET ROLE gbf_bot_cleanup;

-- ✅ 成功すべき操作
DELETE FROM worker.battle_recruitments WHERE id = 1;
DELETE FROM worker.notifications WHERE id = 1;
DELETE FROM worker.scheduled_tasks WHERE id = 1;

-- ❌ 失敗すべき操作（権限なし）
INSERT INTO worker.battle_recruitments (...) VALUES (...);  -- 権限エラー
UPDATE worker.battle_recruitments SET ... WHERE id = 1;      -- 権限エラー
DELETE FROM master.quests WHERE id = 1;                      -- 権限エラー
DELETE FROM guild_master.battle_recruitment_schedules WHERE id = 1;  -- 権限エラー
```

## 関連ドキュメント

- [データベースロール使用ガイド](../../database/db_role_usage.md)
- [データクリーンアップシステム設計書](./data_cleanup_system.md)
- [Row Level Security設計](../../database/rls_design.md)
