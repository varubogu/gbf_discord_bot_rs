# スキーマ分割・RLS適用によるセキュリティ強化計画

## 目的

PostgreSQLのスキーマ分割とRow Level Security (RLS)を導入し、以下のセキュリティリスクを軽減する：

1. **SQLインジェクション対策**: 悪意のあるコマンド入力による他ギルドデータへの不正アクセス防止
2. **アプリケーションバグ対策**: guild_id検証漏れによる誤操作防止
3. **マスターデータ保護**: グローバルマスターデータの意図しない改ざん防止
4. **多層防御の実現**: アプリケーション層とDB層の二重のアクセス制御

## アーキテクチャ概要

### スキーマ分類

```
┌─────────────────────────────────────────────┐
│ master スキーマ                             │
│ - グローバルマスターデータ                  │
│ - スプレッドシートから読み書き              │
│ - 基本的に不変なデータ                      │
├─────────────────────────────────────────────┤
│ quests, quest_aliases, battle_styles,       │
│ elements, channel_types,                    │
│ event_schedules, event_schedule_details,    │
│ message_texts, environments                 │
└─────────────────────────────────────────────┘
         ↓ gbf_bot_guild: SELECT のみ
         ↓ gbf_bot_global: CRUD 可能
┌─────────────────────────────────────────────┐
│ guild_master スキーマ (RLS適用)             │
│ - ギルド固有の設定・マスターデータ          │
│ - guild_id による行レベル分離               │
├─────────────────────────────────────────────┤
│ guilds, guild_channels,                     │
│ guild_spreadsheet_exports,                  │
│ guild_spreadsheet_imports,                  │
│ guild_event_schedules (今後追加予定)       │
└─────────────────────────────────────────────┘
         ↓ guild_id による制限
┌─────────────────────────────────────────────┐
│ worker スキーマ (RLS適用)                   │
│ - 頻繁に更新されるトランザクションデータ    │
│ - 通知・募集の実行時データ                  │
├─────────────────────────────────────────────┤
│ battle_recruitments,                        │
│ notifications,                              │
│ notification_rel_battle_recruitments,       │
│ notification_rel_event_schedules,           │
│ last_process_times                          │
└─────────────────────────────────────────────┘
```

### データベースロール設計

| ロール名 | 用途 | 接続タイミング | 権限 |
|---------|------|---------------|------|
| `gbf_bot_system` | スケジュール通知処理 | 定期実行バッチ | 全スキーマ: SELECT/INSERT/UPDATE (RLS無効・全ギルド一括処理) |
| `gbf_bot_guild` | 通常のコマンド実行 | 各ギルドからのコマンド処理時 | master: SELECT のみ<br>guild_master/worker: CRUD (RLS適用) |
| `gbf_bot_global` | 統計・全体管理 | 全ギルド集計・スプレッドシート更新 | 全スキーマ: CRUD (RLS無効) |
| `gbf_bot_migration` | マイグレーション実行 | マイグレーション実行時 | 全スキーマ: DDL権限 + CRUD (RLS BYPASS) |
| `gbf_bot_admin` | 管理操作 | 管理CLI | 全スキーマ: 全権限 (RLS BYPASS) |

### RLSポリシー設計

#### 基本方針

- **guild_master スキーマ**: 全テーブルに RLS 適用（guild_id による分離）
- **worker スキーマ**: 全テーブルに RLS 適用（guild_id または関連経由）
- **master スキーマ**: RLS不要（gbf_bot_guild は SELECT のみ）
- **RLS無効ロール**: `gbf_bot_system`, `gbf_bot_global`, `gbf_bot_migration`, `gbf_bot_admin`

#### ポリシー適用パターン

```sql
-- パターン1: guild_id による直接分離 (guild_master/worker スキーマ)
CREATE POLICY guild_isolation ON guild_master.guilds
    FOR ALL TO gbf_bot_guild
    USING (guild_id = current_setting('app.current_guild_id')::BIGINT);

CREATE POLICY recruitment_isolation ON worker.battle_recruitments
    FOR ALL TO gbf_bot_guild
    USING (guild_id = current_setting('app.current_guild_id')::BIGINT);

-- パターン2: 関連テーブル経由の分離 (notification_rel 系)
CREATE POLICY notification_rel_isolation ON worker.notification_rel_battle_recruitments
    FOR ALL TO gbf_bot_guild
    USING (
        notification_id IN (
            SELECT id FROM worker.notifications
            WHERE guild_id = current_setting('app.current_guild_id')::BIGINT
        )
    );

-- パターン3: RLS無効ロールには制約なし (BYPASSRLS)
-- gbf_bot_system, gbf_bot_global, gbf_bot_migration, gbf_bot_admin
-- → ポリシー作成不要、BYPASSRLS 属性で全データアクセス可能
```

## テーブル分類詳細

### master スキーマ (8テーブル)

**特徴**: グローバルマスターデータ、スプレッドシートから読み書き、基本的に不変

| テーブル名 | 説明 | 主キー |
|-----------|------|--------|
| quests | クエストマスター | id |
| quest_aliases | クエスト別名マスター | quest_id, sequence_no |
| battle_styles | バトルスタイルマスター | id |
| elements | 属性マスター | id |
| channel_types | チャンネルタイプマスター | id |
| event_schedules | イベントスケジュール | id |
| event_schedule_details | イベント詳細スケジュール | id |
| message_texts | メッセージテキストマスター | id |
| environments | グローバル環境変数 | key |

**権限設定**:
- `gbf_bot_system`: SELECT のみ
- `gbf_bot_guild`: SELECT のみ
- `gbf_bot_global`: CRUD 可能（スプレッドシート更新用）
- `gbf_bot_migration`: ALL
- `gbf_bot_admin`: ALL

### guild_master スキーマ (4テーブル + 今後追加)

**特徴**: ギルド固有の設定・マスターデータ、guild_id による RLS 適用

| テーブル名 | 説明 | 主キー | guild_id | RLS |
|-----------|------|--------|----------|-----|
| guilds | ギルド基本情報 | guild_id | ✓ (PK) | ✓ |
| guild_channels | ギルドチャンネル設定 | guild_id, channel_type | ✓ (PK) | ✓ |
| guild_spreadsheet_exports | スプレッドシート出力設定 | guild_id | ✓ (PK) | ✓ |
| guild_spreadsheet_imports | スプレッドシート入力設定 | guild_id | ✓ (PK) | ✓ |
| guild_event_schedules | ギルド独自イベントスケジュール (今後追加予定) | - | ✓ | ✓ |

**RLSポリシー**:
```sql
-- gbf_bot_guild 用
CREATE POLICY guild_isolation ON guild_master.guilds
    FOR ALL TO gbf_bot_guild
    USING (guild_id = current_setting('app.current_guild_id')::BIGINT);

-- gbf_bot_system, gbf_bot_global は BYPASSRLS により全アクセス可能
```

**権限設定**:
- `gbf_bot_system`: CRUD (RLS無効・全ギルドアクセス)
- `gbf_bot_guild`: CRUD (RLS適用・自ギルドのみ)
- `gbf_bot_global`: CRUD (RLS無効・全ギルドアクセス)
- `gbf_bot_migration`: ALL
- `gbf_bot_admin`: ALL

### worker スキーマ (5テーブル)

**特徴**: 頻繁に更新されるトランザクションデータ、guild_id による RLS 適用

| テーブル名 | 説明 | 主キー | guild_id | RLS |
|-----------|------|--------|----------|-----|
| battle_recruitments | 募集情報 | id | ✓ (FK) | ✓ |
| notifications | 通知基本情報 | id | ✓ (FK) | ✓ |
| notification_rel_battle_recruitments | 募集通知関連 | notification_id, battle_recruitment_id | ✓ (間接) | ✓ |
| notification_rel_event_schedules | イベント通知関連 | notification_id, event_schedule_id | ✓ (間接) | ✓ |
| last_process_times | 最終処理時刻 | process_type | なし | 不要 |

**RLSポリシー**:
```sql
-- guild_id を持つテーブル
CREATE POLICY recruitment_isolation ON worker.battle_recruitments
    FOR ALL TO gbf_bot_guild
    USING (guild_id = current_setting('app.current_guild_id')::BIGINT);

CREATE POLICY notification_isolation ON worker.notifications
    FOR ALL TO gbf_bot_guild
    USING (guild_id = current_setting('app.current_guild_id')::BIGINT);

-- 関連テーブル (JOIN経由)
CREATE POLICY notification_rel_br_isolation ON worker.notification_rel_battle_recruitments
    FOR ALL TO gbf_bot_guild
    USING (
        notification_id IN (
            SELECT id FROM worker.notifications
            WHERE guild_id = current_setting('app.current_guild_id')::BIGINT
        )
    );

CREATE POLICY notification_rel_es_isolation ON worker.notification_rel_event_schedules
    FOR ALL TO gbf_bot_guild
    USING (
        notification_id IN (
            SELECT id FROM worker.notifications
            WHERE guild_id = current_setting('app.current_guild_id')::BIGINT
        )
    );

-- last_process_times は guild_id がないためポリシー不要（全ギルド共通）
CREATE POLICY last_process_times_all ON worker.last_process_times
    FOR ALL TO gbf_bot_guild
    USING (true);

-- gbf_bot_system, gbf_bot_global は BYPASSRLS により全アクセス可能
```

**権限設定**:
- `gbf_bot_system`: CRUD (RLS無効・全ギルドアクセス)
- `gbf_bot_guild`: CRUD (RLS適用・自ギルドのみ、last_process_times は全アクセス)
- `gbf_bot_global`: CRUD (RLS無効・全ギルドアクセス)
- `gbf_bot_migration`: ALL
- `gbf_bot_admin`: ALL

## 実装手順

### フェーズ1: スキーマ作成とロール設定

1. **スキーマ作成**
   ```sql
   CREATE SCHEMA master;
   CREATE SCHEMA guild_master;
   CREATE SCHEMA worker;
   ```

2. **ロール作成と BYPASSRLS 設定**
   ```sql
   CREATE ROLE gbf_bot_system BYPASSRLS LOGIN PASSWORD 'xxx';
   CREATE ROLE gbf_bot_guild LOGIN PASSWORD 'xxx';  -- RLS適用
   CREATE ROLE gbf_bot_global BYPASSRLS LOGIN PASSWORD 'xxx';
   CREATE ROLE gbf_bot_migration BYPASSRLS LOGIN PASSWORD 'xxx';
   CREATE ROLE gbf_bot_admin BYPASSRLS LOGIN PASSWORD 'xxx';
   ```

3. **スキーマレベル権限設定**
   ```sql
   -- master スキーマ
   GRANT USAGE ON SCHEMA master TO gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_migration, gbf_bot_admin;

   GRANT SELECT ON ALL TABLES IN SCHEMA master TO gbf_bot_system, gbf_bot_guild;
   GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA master TO gbf_bot_global;
   GRANT ALL ON ALL TABLES IN SCHEMA master TO gbf_bot_migration, gbf_bot_admin;

   -- guild_master スキーマ
   GRANT USAGE ON SCHEMA guild_master TO gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_migration, gbf_bot_admin;

   GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA guild_master TO gbf_bot_system, gbf_bot_guild, gbf_bot_global;
   GRANT ALL ON ALL TABLES IN SCHEMA guild_master TO gbf_bot_migration, gbf_bot_admin;

   -- worker スキーマ
   GRANT USAGE ON SCHEMA worker TO gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_migration, gbf_bot_admin;

   GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA worker TO gbf_bot_system, gbf_bot_guild, gbf_bot_global;
   GRANT ALL ON ALL TABLES IN SCHEMA worker TO gbf_bot_migration, gbf_bot_admin;
   ```

4. **シーケンス権限設定**
   ```sql
   -- master スキーマ
   GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA master TO gbf_bot_system, gbf_bot_guild, gbf_bot_global;
   GRANT ALL ON ALL SEQUENCES IN SCHEMA master TO gbf_bot_migration, gbf_bot_admin;

   -- guild_master スキーマ
   GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA guild_master TO gbf_bot_system, gbf_bot_guild, gbf_bot_global;
   GRANT ALL ON ALL SEQUENCES IN SCHEMA guild_master TO gbf_bot_migration, gbf_bot_admin;

   -- worker スキーマ
   GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA worker TO gbf_bot_system, gbf_bot_guild, gbf_bot_global;
   GRANT ALL ON ALL SEQUENCES IN SCHEMA worker TO gbf_bot_migration, gbf_bot_admin;
   ```

### フェーズ2: テーブル移動

1. **master スキーマへの移動**
   ```sql
   ALTER TABLE quests SET SCHEMA master;
   ALTER TABLE quest_aliases SET SCHEMA master;
   ALTER TABLE battle_styles SET SCHEMA master;
   ALTER TABLE elements SET SCHEMA master;
   ALTER TABLE channel_types SET SCHEMA master;
   ALTER TABLE event_schedules SET SCHEMA master;
   ALTER TABLE event_schedule_details SET SCHEMA master;
   ALTER TABLE message_texts SET SCHEMA master;
   ALTER TABLE environments SET SCHEMA master;
   ```

2. **guild_master スキーマへの移動**
   ```sql
   ALTER TABLE guilds SET SCHEMA guild_master;
   ALTER TABLE guild_channels SET SCHEMA guild_master;
   ALTER TABLE guild_spreadsheet_exports SET SCHEMA guild_master;
   ALTER TABLE guild_spreadsheet_imports SET SCHEMA guild_master;
   ```

3. **worker スキーマへの移動**
   ```sql
   ALTER TABLE battle_recruitments SET SCHEMA worker;
   ALTER TABLE notifications SET SCHEMA worker;
   ALTER TABLE notification_rel_battle_recruitments SET SCHEMA worker;
   ALTER TABLE notification_rel_event_schedules SET SCHEMA worker;
   ALTER TABLE last_process_times SET SCHEMA worker;
   ```

4. **シーケンス移動**
   ```sql
   -- master スキーマ
   ALTER SEQUENCE quests_id_seq SET SCHEMA master;
   ALTER SEQUENCE battle_styles_id_seq SET SCHEMA master;
   ALTER SEQUENCE elements_id_seq SET SCHEMA master;
   ALTER SEQUENCE channel_types_id_seq SET SCHEMA master;
   ALTER SEQUENCE event_schedules_id_seq SET SCHEMA master;
   ALTER SEQUENCE event_schedule_details_id_seq SET SCHEMA master;
   ALTER SEQUENCE message_texts_id_seq SET SCHEMA master;

   -- worker スキーマ
   ALTER SEQUENCE battle_recruitments_id_seq SET SCHEMA worker;
   ALTER SEQUENCE notifications_id_seq SET SCHEMA worker;
   ```

5. **外部キー制約の再作成**

   スキーマ移動後、外部キーはスキーマ名を含む形式に自動更新されます。
   確認のみ実施：
   ```sql
   -- worker.battle_recruitments の外部キー確認
   SELECT
       tc.constraint_name,
       tc.table_schema,
       tc.table_name,
       kcu.column_name,
       ccu.table_schema AS foreign_table_schema,
       ccu.table_name AS foreign_table_name,
       ccu.column_name AS foreign_column_name
   FROM information_schema.table_constraints AS tc
   JOIN information_schema.key_column_usage AS kcu
       ON tc.constraint_name = kcu.constraint_name
       AND tc.table_schema = kcu.table_schema
   JOIN information_schema.constraint_column_usage AS ccu
       ON ccu.constraint_name = tc.constraint_name
       AND ccu.table_schema = tc.table_schema
   WHERE tc.constraint_type = 'FOREIGN KEY'
       AND tc.table_name = 'battle_recruitments'
       AND tc.table_schema = 'worker';
   ```

### フェーズ3: RLS適用

1. **guild_master スキーマの RLS 有効化**
   ```sql
   ALTER TABLE guild_master.guilds ENABLE ROW LEVEL SECURITY;
   ALTER TABLE guild_master.guild_channels ENABLE ROW LEVEL SECURITY;
   ALTER TABLE guild_master.guild_spreadsheet_exports ENABLE ROW LEVEL SECURITY;
   ALTER TABLE guild_master.guild_spreadsheet_imports ENABLE ROW LEVEL SECURITY;

   -- FORCE RLS (SUPERUSER でも RLS 適用)
   ALTER TABLE guild_master.guilds FORCE ROW LEVEL SECURITY;
   ALTER TABLE guild_master.guild_channels FORCE ROW LEVEL SECURITY;
   ALTER TABLE guild_master.guild_spreadsheet_exports FORCE ROW LEVEL SECURITY;
   ALTER TABLE guild_master.guild_spreadsheet_imports FORCE ROW LEVEL SECURITY;
   ```

2. **guild_master スキーマのポリシー作成**
   ```sql
   -- guilds
   CREATE POLICY guilds_isolation ON guild_master.guilds
       FOR ALL TO gbf_bot_guild
       USING (guild_id = current_setting('app.current_guild_id')::BIGINT);

   -- guild_channels
   CREATE POLICY guild_channels_isolation ON guild_master.guild_channels
       FOR ALL TO gbf_bot_guild
       USING (guild_id = current_setting('app.current_guild_id')::BIGINT);

   -- guild_spreadsheet_exports
   CREATE POLICY guild_spreadsheet_exports_isolation ON guild_master.guild_spreadsheet_exports
       FOR ALL TO gbf_bot_guild
       USING (guild_id = current_setting('app.current_guild_id')::BIGINT);

   -- guild_spreadsheet_imports
   CREATE POLICY guild_spreadsheet_imports_isolation ON guild_master.guild_spreadsheet_imports
       FOR ALL TO gbf_bot_guild
       USING (guild_id = current_setting('app.current_guild_id')::BIGINT);
   ```

3. **worker スキーマの RLS 有効化**
   ```sql
   ALTER TABLE worker.battle_recruitments ENABLE ROW LEVEL SECURITY;
   ALTER TABLE worker.notifications ENABLE ROW LEVEL SECURITY;
   ALTER TABLE worker.notification_rel_battle_recruitments ENABLE ROW LEVEL SECURITY;
   ALTER TABLE worker.notification_rel_event_schedules ENABLE ROW LEVEL SECURITY;
   ALTER TABLE worker.last_process_times ENABLE ROW LEVEL SECURITY;

   -- FORCE RLS
   ALTER TABLE worker.battle_recruitments FORCE ROW LEVEL SECURITY;
   ALTER TABLE worker.notifications FORCE ROW LEVEL SECURITY;
   ALTER TABLE worker.notification_rel_battle_recruitments FORCE ROW LEVEL SECURITY;
   ALTER TABLE worker.notification_rel_event_schedules FORCE ROW LEVEL SECURITY;
   ALTER TABLE worker.last_process_times FORCE ROW LEVEL SECURITY;
   ```

4. **worker スキーマのポリシー作成**
   ```sql
   -- battle_recruitments
   CREATE POLICY battle_recruitments_isolation ON worker.battle_recruitments
       FOR ALL TO gbf_bot_guild
       USING (guild_id = current_setting('app.current_guild_id')::BIGINT);

   -- notifications
   CREATE POLICY notifications_isolation ON worker.notifications
       FOR ALL TO gbf_bot_guild
       USING (guild_id = current_setting('app.current_guild_id')::BIGINT);

   -- notification_rel_battle_recruitments (JOIN経由)
   CREATE POLICY notification_rel_br_isolation ON worker.notification_rel_battle_recruitments
       FOR ALL TO gbf_bot_guild
       USING (
           notification_id IN (
               SELECT id FROM worker.notifications
               WHERE guild_id = current_setting('app.current_guild_id')::BIGINT
           )
       );

   -- notification_rel_event_schedules (JOIN経由)
   CREATE POLICY notification_rel_es_isolation ON worker.notification_rel_event_schedules
       FOR ALL TO gbf_bot_guild
       USING (
           notification_id IN (
               SELECT id FROM worker.notifications
               WHERE guild_id = current_setting('app.current_guild_id')::BIGINT
           )
       );

   -- last_process_times (guild_id なし、全ギルド共通)
   CREATE POLICY last_process_times_all ON worker.last_process_times
       FOR ALL TO gbf_bot_guild
       USING (true);
   ```

### フェーズ4: デフォルト権限設定

新規テーブル作成時の自動権限付与：

```sql
-- master スキーマ
ALTER DEFAULT PRIVILEGES IN SCHEMA master
    GRANT SELECT ON TABLES TO gbf_bot_system, gbf_bot_guild;

ALTER DEFAULT PRIVILEGES IN SCHEMA master
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO gbf_bot_global;

ALTER DEFAULT PRIVILEGES IN SCHEMA master
    GRANT ALL ON TABLES TO gbf_bot_migration, gbf_bot_admin;

-- guild_master スキーマ
ALTER DEFAULT PRIVILEGES IN SCHEMA guild_master
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO gbf_bot_system, gbf_bot_guild, gbf_bot_global;

ALTER DEFAULT PRIVILEGES IN SCHEMA guild_master
    GRANT ALL ON TABLES TO gbf_bot_migration, gbf_bot_admin;

-- worker スキーマ
ALTER DEFAULT PRIVILEGES IN SCHEMA worker
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO gbf_bot_system, gbf_bot_guild, gbf_bot_global;

ALTER DEFAULT PRIVILEGES IN SCHEMA worker
    GRANT ALL ON TABLES TO gbf_bot_migration, gbf_bot_admin;

-- シーケンス
ALTER DEFAULT PRIVILEGES IN SCHEMA master, guild_master, worker
    GRANT USAGE, SELECT ON SEQUENCES TO gbf_bot_system, gbf_bot_guild, gbf_bot_global;

ALTER DEFAULT PRIVILEGES IN SCHEMA master, guild_master, worker
    GRANT ALL ON SEQUENCES TO gbf_bot_migration, gbf_bot_admin;
```

### フェーズ5: アプリケーション対応

1. **接続ロール変更**
   ```bash
   # .env
   # データベース接続情報（共通）
   DB_HOST=localhost
   DB_PORT=5432
   DB_NAME=gbf_bot_db

   # 通常のコマンド実行用（Guildロール）
   GUILD_DB_USER=gbf_bot_guild
   GUILD_DB_PASSWORD=your_guild_password

   # スケジューラー実行用（Systemロール）
   SYSTEM_DB_USER=gbf_bot_system
   SYSTEM_DB_PASSWORD=your_system_password

   # 統計・管理処理用（Globalロール）
   GLOBAL_DB_USER=gbf_bot_global
   GLOBAL_DB_PASSWORD=your_global_password

   # マイグレーション実行用（Adminロール）
   ADMIN_DB_USER=gbf_bot_admin
   ADMIN_DB_PASSWORD=your_admin_password
   ```

2. **セッション変数設定用ヘルパー関数（Repository層）**
   ```rust
   // src/repository/db_helper.rs (新規作成)
   use sea_orm::{DatabaseConnection, DatabaseTransaction, DbErr, Statement, ConnectionTrait};
   use sea_orm::DatabaseBackend;

   /// 現在のギルドIDをセッション変数に設定
   ///
   /// RLSポリシーで使用されるため、トランザクション開始直後に必ず呼び出す
   pub async fn set_current_guild_id<C>(
       conn: &C,
       guild_id: i64,
   ) -> Result<(), DbErr>
   where
       C: ConnectionTrait,
   {
       conn.execute(Statement::from_string(
           DatabaseBackend::Postgres,
           format!("SET LOCAL app.current_guild_id = {}", guild_id),
       ))
       .await?;
       Ok(())
   }
   ```

3. **Facade層での呼び出し**
   ```rust
   // src/facades/battle_recruitment_facade.rs (例)
   use crate::repository::db_helper::set_current_guild_id;

   pub async fn create_recruitment(
       app_state: &AppState,
       guild_id: i64,
       // ... 他のパラメータ
   ) -> Result<Recruitment, FacadeError> {
       let txn = app_state.db().begin().await?;

       // トランザクション開始直後に guild_id を設定
       set_current_guild_id(&txn, guild_id).await?;

       // 以降の処理は RLS により自動的に guild_id でフィルタされる
       let result = async {
           let recruitment = recruitment_service
               .create(&txn, /* ... */)
               .await?;

           Ok(recruitment)
       }
       .await;

       match result {
           Ok(recruitment) => {
               txn.commit().await?;
               Ok(recruitment)
           }
           Err(e) => {
               txn.rollback().await?;
               Err(e)
           }
       }
   }
   ```

4. **Entity のスキーマ指定**
   ```rust
   // src/models/entities/quests.rs
   use sea_orm::entity::prelude::*;

   #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
   #[sea_orm(schema_name = "master", table_name = "quests")]
   pub struct Model {
       #[sea_orm(primary_key)]
       pub id: i32,
       // ...
   }

   // src/models/entities/guilds.rs
   #[sea_orm(schema_name = "guild_master", table_name = "guilds")]
   pub struct Model {
       #[sea_orm(primary_key, auto_increment = false)]
       pub guild_id: i64,
       // ...
   }

   // src/models/entities/battle_recruitments.rs
   #[sea_orm(schema_name = "worker", table_name = "battle_recruitments")]
   pub struct Model {
       #[sea_orm(primary_key)]
       pub id: i32,
       pub guild_id: i64,
       // ...
   }
   ```

5. **外部キー参照の更新**
   ```rust
   // src/models/entities/battle_recruitments.rs
   #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
   pub enum Relation {
       // スキーマをまたぐ外部キー
       #[sea_orm(
           belongs_to = "super::quests::Entity",
           from = "Column::QuestId",
           to = "super::quests::Column::Id"
       )]
       Quest,

       #[sea_orm(
           belongs_to = "super::guilds::Entity",
           from = "Column::GuildId",
           to = "super::guilds::Column::GuildId"
       )]
       Guild,
   }
   ```

   **注**: SeaORM はスキーマ名を自動的に処理するため、Relation 定義の変更は不要です。

## マイグレーション戦略

### マイグレーションファイル構成

1. **m20251127_000000_create_schemas_and_roles.rs**
   - スキーマ作成
   - ロール作成（パスワードは環境変数から取得）
   - BYPASSRLS 設定

2. **m20251127_000001_set_schema_permissions.rs**
   - スキーマレベル権限設定
   - シーケンス権限設定

3. **m20251127_000002_move_tables_to_schemas.rs**
   - テーブルのスキーマ移動
   - シーケンス移動
   - 外部キー制約の確認

4. **m20251127_000003_enable_row_level_security.rs**
   - RLS有効化
   - FORCE RLS設定
   - ポリシー作成

5. **m20251127_000004_set_default_privileges.rs**
   - デフォルト権限設定

### ロールバック対応

各マイグレーションの `down()` で元の状態に戻せるようにする：

```rust
async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    // RLS無効化
    manager
        .get_connection()
        .execute_unprepared("ALTER TABLE guild_master.guilds DISABLE ROW LEVEL SECURITY")
        .await?;

    // テーブルを public スキーマに戻す
    manager
        .get_connection()
        .execute_unprepared("ALTER TABLE master.quests SET SCHEMA public")
        .await?;

    // ポリシー削除
    manager
        .get_connection()
        .execute_unprepared("DROP POLICY IF EXISTS guild_isolation ON guild_master.guilds")
        .await?;

    Ok(())
}
```

## セキュリティ検証項目

### テストシナリオ

#### 1. 正常系: guild_id による分離

```sql
-- gbf_bot_guild ロールで接続
SET ROLE gbf_bot_guild;
SET app.current_guild_id = 123;

-- 自ギルドのデータのみ取得
SELECT * FROM guild_master.guilds;
-- 結果: guild_id=123 のレコードのみ

SELECT * FROM worker.battle_recruitments;
-- 結果: guild_id=123 のレコードのみ
```

#### 2. 異常系: 他ギルドアクセス拒否

```sql
-- gbf_bot_guild ロールで接続
SET ROLE gbf_bot_guild;
SET app.current_guild_id = 123;

-- 他ギルドのデータは取得不可
SELECT * FROM guild_master.guilds WHERE guild_id = 456;
-- 結果: 0件（RLSにより自動フィルタ）

-- 他ギルドのデータ更新も不可
UPDATE guild_master.guilds SET name = 'hacked' WHERE guild_id = 456;
-- 結果: 0 rows updated（RLSにより対象外）
```

#### 3. 異常系: マスターデータ更新拒否

```sql
-- gbf_bot_guild ロールで接続
SET ROLE gbf_bot_guild;

-- SELECT は可能
SELECT * FROM master.quests;
-- 結果: 成功

-- INSERT/UPDATE/DELETE は権限エラー
INSERT INTO master.quests (name, ...) VALUES ('hacked', ...);
-- エラー: permission denied for table quests
```

#### 4. 正常系: スケジューラーでの全ギルドアクセス

```sql
-- gbf_bot_system ロールで接続
SET ROLE gbf_bot_system;

-- RLS無効のため全ギルド取得可能
SELECT COUNT(*) FROM guild_master.guilds;
-- 結果: 全ギルド数

-- 全ギルドの募集を一括処理可能
UPDATE worker.battle_recruitments
SET is_recruiting = false
WHERE quest_start_at < NOW();
-- 結果: 全ギルドのレコードが更新される
```

#### 5. 正常系: グローバルロールでのCRUD

```sql
-- gbf_bot_global ロールで接続
SET ROLE gbf_bot_global;

-- RLS無効のため全ギルド参照可能
SELECT guild_id, COUNT(*) FROM worker.battle_recruitments GROUP BY guild_id;
-- 結果: 全ギルドの集計結果

-- マスターデータの更新可能
UPDATE master.quests SET name = 'Updated Quest Name' WHERE id = 1;
-- 結果: 成功
```

### 自動テストの実装

```rust
// tests/rls_security_test.rs
#[tokio::test]
async fn test_rls_guild_isolation() {
    // gbf_bot_guild ロールで接続
    let db = Database::connect("postgresql://gbf_bot_guild:xxx@localhost/gbf_bot")
        .await
        .unwrap();

    let txn = db.begin().await.unwrap();

    // guild_id=123 を設定
    set_current_guild_id(&txn, 123).await.unwrap();

    // 自ギルドのデータのみ取得
    let guilds = Guilds::find().all(&txn).await.unwrap();
    assert_eq!(guilds.len(), 1);
    assert_eq!(guilds[0].guild_id, 123);

    txn.rollback().await.unwrap();
}

#[tokio::test]
async fn test_rls_prevents_other_guild_access() {
    let db = Database::connect("postgresql://gbf_bot_guild:xxx@localhost/gbf_bot")
        .await
        .unwrap();

    let txn = db.begin().await.unwrap();
    set_current_guild_id(&txn, 123).await.unwrap();

    // 他ギルドのデータは取得不可
    let other_guild = Guilds::find_by_id(456).one(&txn).await.unwrap();
    assert!(other_guild.is_none());

    txn.rollback().await.unwrap();
}
```

## 運用上の注意点

### パフォーマンス

- **RLSのオーバーヘッド**: セッション変数チェックのみのため軽微（1-2%程度）
- **インデックス**: `guild_id` にインデックス作成済み（FK制約で自動作成）
- **統計情報**: RLS適用後は `ANALYZE` を実行
  ```sql
  ANALYZE guild_master.guilds;
  ANALYZE worker.battle_recruitments;
  ```

### 開発環境

- **ローカルDB**: 同じロール構成を作成
- **マイグレーション**: `gbf_bot_admin` ロールで実行（環境変数から自動選択）
  ```bash
  # .envファイルに以下を設定
  # ADMIN_DB_USER=gbf_bot_admin
  # ADMIN_DB_PASSWORD=your_admin_password
  cargo run -- migrate
  ```
- **手動クエリ**: `SET ROLE` でテスト可能
  ```sql
  SET ROLE gbf_bot_guild;
  SET app.current_guild_id = 123;
  SELECT * FROM guild_master.guilds;
  ```

### 本番環境

#### デプロイ手順

1. **マイグレーション実行**（ダウンタイムなし）
   ```bash
   # 環境変数を設定してマイグレーション実行
   # ADMIN_DB_USER=gbf_bot_admin
   # ADMIN_DB_PASSWORD=xxx
   cargo run -- migrate
   ```

2. **アプリケーションデプロイ**（接続ロール変更）
   - 新しいバイナリに `.env` で各ロールの接続情報を設定
   - ローリングデプロイ可能（スキーマ変更は後方互換）

3. **動作確認**
   - セッション変数設定の動作確認
   - RLSポリシーの動作確認

#### ロールバック手順

1. アプリケーションを旧バージョンにロールバック（`public` スキーマに戻す）
2. マイグレーション `down()` 実行
   ```bash
   # ADMIN_DB_USER/ADMIN_DB_PASSWORD環境変数を設定
   sea-orm-cli migrate down
   ```

## 今後の拡張

### guild固有設定の追加

将来的に `guild_event_schedules`、`guild_environments` などを追加する場合：

1. **テーブル作成**（guild_master スキーマ）
   ```sql
   CREATE TABLE guild_master.guild_event_schedules (
       guild_id BIGINT NOT NULL,
       event_id INT NOT NULL,
       start_time TIMESTAMPTZ NOT NULL,
       -- ...
       PRIMARY KEY (guild_id, event_id),
       FOREIGN KEY (guild_id) REFERENCES guild_master.guilds(guild_id)
   );
   ```

2. **RLS適用**（自動的にデフォルト権限が適用される）
   ```sql
   ALTER TABLE guild_master.guild_event_schedules ENABLE ROW LEVEL SECURITY;
   ALTER TABLE guild_master.guild_event_schedules FORCE ROW LEVEL SECURITY;

   CREATE POLICY guild_event_schedules_isolation ON guild_master.guild_event_schedules
       FOR ALL TO gbf_bot_guild
       USING (guild_id = current_setting('app.current_guild_id')::BIGINT);
   ```

3. **優先順位ロジック**（アプリケーション層）
   ```rust
   // guild固有設定を優先、なければグローバル設定
   pub async fn get_event_schedule(
       db: &DatabaseTransaction,
       guild_id: i64,
       event_id: i32,
   ) -> Result<EventSchedule, Error> {
       // guild固有設定を検索
       if let Some(guild_schedule) = GuildEventSchedules::find_by_id((guild_id, event_id))
           .one(db)
           .await?
       {
           return Ok(guild_schedule.into());
       }

       // グローバル設定を使用
       EventSchedules::find_by_id(event_id)
           .one(db)
           .await?
           .ok_or(Error::NotFound)
           .map(|s| s.into())
   }
   ```

## リスク評価

| リスク | 発生確率 | 影響度 | 対策 |
|--------|---------|--------|------|
| マイグレーション失敗 | 低 | 高 | ステージング環境で事前検証、ロールバック手順の準備 |
| RLSポリシー設定ミス | 中 | 高 | 自動テスト実装、手動検証チェックリスト |
| パフォーマンス劣化 | 低 | 中 | ベンチマーク実施、インデックス確認 |
| セッション変数設定漏れ | 中 | 高 | 統合テスト実装、Facade層での一貫した設定 |
| ロール権限不足エラー | 中 | 中 | 権限確認スクリプト、エラーログ監視 |

## まとめ

このスキーマ分割・RLS適用により、以下を実現：

1. ✅ **セキュリティ強化**: SQLインジェクション・アプリケーションバグへの多層防御
2. ✅ **保守性向上**: 論理的なテーブル分類による可読性向上（master/guild_master/worker）
3. ✅ **運用安全性**: マスターデータの誤操作防止（gbf_bot_guild は SELECT のみ）
4. ✅ **柔軟な権限管理**: ロールごとの細かい権限制御（scheduler/guild/global/migration/admin）
5. ✅ **拡張性**: guild固有設定への対応準備（guild_master スキーマ）

次のステップとして、マイグレーションファイルの実装を進めます。
