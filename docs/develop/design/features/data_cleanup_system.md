# データクリーンアップシステム設計書

## 概要

GBF Discord Botのデータベースに蓄積された過去のデータを定期的に削除し、DB肥大化を防ぐシステムの設計書。
Botプロセスとは独立した専用メンテナンスコンテナとして実装し、毎日深夜帯にcronで自動実行される。

## 背景と目的

### 課題
- マルチ募集、通知、スケジュールタスクなどのデータが日々蓄積される
- 古いデータが削除されずに残り続けると、DB容量が増大し続ける
- クエリパフォーマンスの低下やバックアップ時間の増加につながる
- Botプロセス内で実行すると、負荷が集中しユーザー操作に影響する可能性がある

### 目的
- 定期的に古いデータを自動削除し、DB容量を適切に管理する
- ユーザーの少ない深夜帯に実行し、サービスへの影響を最小化する
- Botプロセスとは独立したメンテナンスコンテナとして実装し、負荷を分散する
- 既存のRustコードベース（Repository層）を再利用し、保守性を高める

## 設計原則

### 1. Botプロセスからの独立
- Botコンテナとは別の専用メンテナンスコンテナとして実装
- cron定期実行により、Bot負荷と完全に分離
- 障害時もBot本体に影響を与えない

### 2. 既存コードベースの再利用
- Rustで実装し、既存のRepository層、エンティティ、エラーハンドリング機構を再利用
- SeaORM、tracing、thiserrorなどの既存ライブラリをそのまま活用
- 保守性とコードの一貫性を保つ

### 3. 安全性重視
- トランザクション内で削除を実行し、エラー時は自動ロールバック
- 削除対象は明確な基準（日数、フラグ）で判定
- 冪等性を保証し、複数回実行しても同じ結果になる

### 4. シンプルな実行フロー
- cron起動 → データ削除 → 終了というシンプルな流れ
- scheduled_tasksテーブルへの登録は不要（Botと独立しているため）
- 実行履歴はログファイルとして保存

### 5. 保守性と拡張性
- 削除対象テーブルの追加・変更が容易
- クリーンアップロジックは独立したバイナリに集約
- 削除基準（日数、フラグ）は環境変数や設定ファイルで外部化可能

## システム構成

### アーキテクチャ

```
┌─────────────────────────────────────┐
│  Maintenanceコンテナ (cron起動)      │
│                                     │
│  src/bin/cleanup.rs                 │
│    ↓                                │
│  DataCleanupService                 │
│    ├─ cleanup_before計算            │
│    ├─ トランザクション開始          │
│    ├─ 各テーブル削除実行            │
│    └─ コミット/ロールバック         │
│    ↓                                │
│  Repository層（既存コード再利用）    │
│    ├─ BattleRecruitmentRepository   │
│    ├─ NotificationRepository        │
│    ├─ ScheduledTaskRepository       │
│    └─ 各種Repository                │
│    ↓                                │
│  PostgreSQL (DB)                    │
└─────────────────────────────────────┘

┌─────────────────────────────────────┐
│  Botコンテナ (常駐)                  │
│                                     │
│  ← Maintenanceとは完全に独立        │
│  ← 削除処理の影響を受けない         │
└─────────────────────────────────────┘
```

### コンポーネント設計

#### src/bin/cleanup.rs (メインバイナリ)

**責務:**
- メンテナンスバッチのエントリーポイント
- 環境変数からDB接続情報を取得
- DataCleanupServiceを初期化・実行
- 実行結果をログ出力して終了

**実装例:**
```rust
use gbf_discord_bot_rs::config::Config;
use gbf_discord_bot_rs::database::Database;
use gbf_discord_bot_rs::services::maintenance::DataCleanupService;
use tracing::{info, error};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ログ初期化
    tracing_subscriber::fmt::init();

    info!("データクリーンアップバッチを開始します");

    // 設定読み込み
    let config = Config::from_env()?;

    // DB接続
    let db = Database::connect(&config.database_url).await?;

    // クリーンアップサービス初期化
    let cleanup_service = DataCleanupService::new(db);

    // クリーンアップ実行
    match cleanup_service.execute().await {
        Ok(stats) => {
            info!(
                recruitments = stats.deleted_recruitments,
                notifications = stats.deleted_notifications,
                tasks = stats.deleted_tasks,
                "データクリーンアップが正常に完了しました"
            );
            Ok(())
        }
        Err(e) => {
            error!(error = %e, "データクリーンアップに失敗しました");
            Err(e.into())
        }
    }
}
```

#### DataCleanupService

**責務:**
- クリーンアップ処理全体の制御
- 削除基準日時の計算（実行日時 - 30日）
- トランザクション管理
- 各Repository呼び出し

**主要メソッド:**
```rust
pub struct DataCleanupService {
    db: DatabaseConnection,
    retention_days: i64,  // 環境変数から取得（デフォルト30日）
}

pub struct CleanupStatistics {
    pub deleted_recruitments: u64,
    pub deleted_notifications: u64,
    pub deleted_tasks: u64,
    pub cleanup_before: DateTime<Utc>,
}

impl DataCleanupService {
    pub fn new(db: DatabaseConnection) -> Self {
        let retention_days = std::env::var("CLEANUP_RETENTION_DAYS")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .unwrap_or(30);

        Self { db, retention_days }
    }

    /// データクリーンアップを実行
    pub async fn execute(&self) -> Result<CleanupStatistics> {
        info!("データクリーンアップを開始します");

        // 削除基準日時を計算（現在時刻 - 保持期間）
        let cleanup_before = Utc::now() - Duration::days(self.retention_days);
        info!(cleanup_before = %cleanup_before, "削除基準日時を計算しました");

        // トランザクション開始
        let txn = self.db.begin().await?;

        // 各テーブルのクリーンアップを実行
        let deleted_recruitments = self.cleanup_battle_recruitments(&txn, cleanup_before).await?;
        let deleted_notifications = self.cleanup_notifications(&txn, cleanup_before).await?;
        let deleted_tasks = self.cleanup_scheduled_tasks(&txn, cleanup_before).await?;

        // コミット
        txn.commit().await?;

        Ok(CleanupStatistics {
            deleted_recruitments,
            deleted_notifications,
            deleted_tasks,
            cleanup_before,
        })
    }

    /// battle_recruitmentsテーブルのクリーンアップ
    async fn cleanup_battle_recruitments(
        &self,
        txn: &DatabaseTransaction,
        cleanup_before: DateTime<Utc>,
    ) -> Result<u64> {
        // Repository層を利用して削除
        let result = BattleRecruitments::delete_many()
            .filter(battle_recruitments::Column::QuestStartAt.lt(cleanup_before))
            .filter(battle_recruitments::Column::IsRecruiting.eq(false))
            .exec(txn)
            .await?;

        info!(deleted_count = result.rows_affected, "battle_recruitmentsを削除しました");
        Ok(result.rows_affected)
    }

    /// notificationsテーブルのクリーンアップ
    async fn cleanup_notifications(
        &self,
        txn: &DatabaseTransaction,
        cleanup_before: DateTime<Utc>,
    ) -> Result<u64> {
        let result = Notifications::delete_many()
            .filter(notifications::Column::ScheduleDatetime.lt(cleanup_before))
            .filter(notifications::Column::IsSent.eq(true))
            .exec(txn)
            .await?;

        info!(deleted_count = result.rows_affected, "notificationsを削除しました");
        Ok(result.rows_affected)
    }

    /// scheduled_tasksテーブルのクリーンアップ
    async fn cleanup_scheduled_tasks(
        &self,
        txn: &DatabaseTransaction,
        cleanup_before: DateTime<Utc>,
    ) -> Result<u64> {
        let result = ScheduledTasks::delete_many()
            .filter(scheduled_tasks::Column::ScheduleDatetime.lt(cleanup_before))
            .filter(scheduled_tasks::Column::IsExecuted.eq(true))
            .filter(scheduled_tasks::Column::TaskType.ne(3))  // DataCleanup以外
            .exec(txn)
            .await?;

        info!(deleted_count = result.rows_affected, "scheduled_tasksを削除しました");
        Ok(result.rows_affected)
    }
}
```

## データモデル

### テーブル構成の変更

**scheduled_task_cleanups テーブルは使用しません**

- メンテナンスコンテナは scheduled_tasks テーブルに依存せず、完全に独立して動作
- 削除基準日時は実行時に計算（`Utc::now() - Duration::days(30)`）
- scheduled_tasks テーブルへのタスク登録は不要（cron起動のため）

この設計により、以下のメリットがあります：
- Bot本体とメンテナンスバッチが完全に独立
- scheduled_tasks テーブルの肥大化を防ぐ
- シンプルな実装（テーブル登録・削除の処理が不要）

## 削除対象テーブルとデータ

### 削除対象と基準

| テーブル名 | 削除基準 | 保持期間 | CASCADE削除される関連テーブル |
|-----------|---------|---------|---------------------------|
| **battle_recruitments** | `quest_start_at < cleanup_before` AND `is_recruiting = false` | 30日 | `recruitment_participants`<br>`battle_recruitment_dismissals`<br>`notification_rel_battle_recruitments`<br>`scheduled_task_dissolutions`<br>`scheduled_task_dismissals` |
| **notifications** | `schedule_datetime < cleanup_before` AND `is_sent = true` | 30日 | `notification_rel_battle_recruitments`<br>`notification_rel_event_schedules`<br>`scheduled_task_notifications` |
| **scheduled_tasks** | `schedule_datetime < cleanup_before` AND `is_executed = true` AND `task_type != 3` | 30日 | `scheduled_task_notifications`<br>`scheduled_task_dissolutions`<br>`scheduled_task_dismissals`<br>`scheduled_task_recurring_recruitments`<br>`scheduled_task_cleanups` |

**注:**
- `cleanup_before` = 実行日時 - 30日
- 未実行・募集中・未送信のデータは削除しない（将来実行される可能性があるため）
- `task_type = 3` (DataCleanup) のタスク自体は削除しない（連鎖的に削除されることを防ぐ）

### 削除されるデータの詳細

#### 1. battle_recruitments（マルチ募集）
**削除条件:**
- クエスト開始日時が30日以上前
- 募集が終了している（`is_recruiting = false`）

**CASCADE削除される関連データ:**
- `recruitment_participants`: 参加者記録
- `battle_recruitment_dismissals`: 解散時刻設定
- `notification_rel_battle_recruitments`: 通知との関連付け
- `scheduled_task_dissolutions`: 解散タスク
- `scheduled_task_dismissals`: 人数不足解散タスク

#### 2. notifications（通知）
**削除条件:**
- 通知予定日時が30日以上前
- 通知が送信済み（`is_sent = true`）

**CASCADE削除される関連データ:**
- `notification_rel_battle_recruitments`: 募集との関連付け
- `notification_rel_event_schedules`: イベントスケジュールとの関連付け
- `scheduled_task_notifications`: 通知タスク

#### 3. scheduled_tasks（スケジュールタスク履歴）
**削除条件:**
- 実行予定日時が30日以上前
- 実行済み（`is_executed = true`）
- DataCleanupタスク以外（`task_type != 3`）

**CASCADE削除される関連データ:**
- `scheduled_task_notifications`: 通知タスク詳細
- `scheduled_task_dissolutions`: 解散タスク詳細
- `scheduled_task_dismissals`: 人数不足解散タスク詳細
- `scheduled_task_recurring_recruitments`: 定期募集タスク詳細
- `scheduled_task_cleanups`: クリーンアップタスク詳細（通常は削除されない）

### 削除されないデータ（保持されるデータ）

以下のデータは削除対象外として保持される:

#### 1. マスタデータ（永続保持）
- `master.quests`: クエストマスタ
- `master.battle_styles`: 戦闘スタイルマスタ
- `master.elements`: 属性マスタ
- `master.event_schedules`: イベントスケジュール
- `master.event_schedule_details`: イベントスケジュール詳細

#### 2. ギルド設定データ（ユーザー削除まで保持）
- `guild_master.battle_recruitment_schedules`: 定期募集テンプレート
- `guild_master.battle_recruitment_schedule_days`: 定期募集曜日設定
- `guild_master.battle_recruitment_schedule_dismissals`: 定期募集解散設定
- `guild_master.guild_event_schedules`: ギルド固有イベントスケジュール
- `guild_master.guild_event_schedule_details`: ギルド固有イベント詳細

#### 3. 実行中・未実行のデータ
- 募集中（`is_recruiting = true`）のマルチ募集
- 未送信（`is_sent = false`）の通知
- 未実行（`is_executed = false`）のスケジュールタスク

#### 4. 最近30日間のデータ
- 30日以内に実行されたすべてのデータ

## 処理フロー

### cron起動フロー（毎日AM3時）

```
cron (AM 3:00 JST)
  ↓
docker compose run --rm maintenance
  ↓
Maintenanceコンテナ起動
  ↓
src/bin/cleanup.rs 実行
  ↓
  ├─ [1] ログ初期化
  │   └─ tracing_subscriber::fmt::init()
  │
  ├─ [2] 環境変数から設定読み込み
  │   ├─ DATABASE_URL
  │   └─ CLEANUP_RETENTION_DAYS (デフォルト: 30)
  │
  ├─ [3] DB接続
  │   └─ Database::connect(&config.database_url)
  │
  ├─ [4] DataCleanupService初期化
  │   └─ DataCleanupService::new(db)
  │
  ├─ [5] クリーンアップ実行
  │   └─ DataCleanupService::execute()
  │       │
  │       ├─ cleanup_before計算
  │       │   └─ Utc::now() - Duration::days(retention_days)
  │       │
  │       ├─ トランザクション開始
  │       │
  │       ├─ 各テーブルクリーンアップ
  │       │   ├─ cleanup_battle_recruitments()
  │       │   │   └─ DELETE battle_recruitments
  │       │   │       WHERE quest_start_at < cleanup_before
  │       │   │         AND is_recruiting = false
  │       │   │
  │       │   ├─ cleanup_notifications()
  │       │   │   └─ DELETE notifications
  │       │   │       WHERE schedule_datetime < cleanup_before
  │       │   │         AND is_sent = true
  │       │   │
  │       │   └─ cleanup_scheduled_tasks()
  │       │       └─ DELETE scheduled_tasks
  │       │           WHERE schedule_datetime < cleanup_before
  │       │             AND is_executed = true
  │       │
  │       └─ トランザクションコミット
  │
  ├─ [6] 統計情報をログ出力
  │   └─ info!(deleted_recruitments, deleted_notifications, deleted_tasks)
  │
  └─ [7] プロセス終了（終了コード: 0=成功, 1=失敗）
  ↓
コンテナ停止・削除 (--rm)
```

### エラー時の挙動

```
DataCleanupService::execute() でエラー発生
  ↓
トランザクション自動ロールバック
  ├─ すべての削除がロールバックされる
  ├─ データの一貫性は保たれる
  │
  └─ エラーログ出力
      └─ error!(error = %e, "データクリーンアップに失敗しました")
  ↓
main() がエラーを返す
  ↓
プロセス終了（終了コード: 1）
  ↓
cron が翌日再度実行
  └─ 翌日AM3時に自動リトライ
```

### 手動実行フロー

管理者が手動で実行する場合:

```bash
# docker-compose.ymlがあるディレクトリで実行
docker compose run --rm maintenance

# または環境変数を指定して実行
docker compose run --rm -e CLEANUP_RETENTION_DAYS=60 maintenance
```

## 実行タイミング

### cron設定

- **実行時刻**: 毎日 AM 3:00 (JST)
- **cron式**: `0 3 * * *` （ホストOSのcrontabまたはcronコンテナで設定）
- **理由**:
  - ユーザーの利用が最も少ない時間帯
  - マルチ募集のピークタイム（夜間）を避ける
  - DBバックアップ時刻との競合を避ける

### cron設定例（ホストOS）

```bash
# crontab -e で編集
# 毎日AM3時（JST）に実行
0 3 * * * cd /path/to/gbf_discord_bot_rs && docker compose run --rm maintenance >> /var/log/cleanup.log 2>&1
```

### cron設定例（cronコンテナ使用）

docker-compose.ymlにcronコンテナを追加する場合:

```yaml
services:
  # ... 既存のapp, db, maintenanceサービス ...

  cron:
    image: alpine:latest
    restart: unless-stopped
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock:ro
      - ./cron/cleanup.sh:/etc/periodic/daily/cleanup:ro
    command: crond -f -l 2
    networks:
      - app_network
```

cron/cleanup.sh:
```bash
#!/bin/sh
cd /app && docker compose run --rm maintenance
```

## Docker構成

### Maintenanceコンテナの定義

docker-compose.ymlに追加:

```yaml
services:
  # ... 既存のapp, dbサービス ...

  maintenance:
    image: ghcr.io/${GITHUB_REPOSITORY:-varubogu/gbf_discord_bot_rs}-maintenance:latest
    # 検証用: ローカルビルド
    # build:
    #   context: .
    #   dockerfile: Dockerfile.maintenance
    env_file:
      - .env.maintenance
    environment:
      DB_HOST: db
      TZ: UTC
    networks:
      - app_network
    depends_on:
      db:
        condition: service_healthy
    profiles:
      - maintenance  # デフォルトでは起動しない
```

### Dockerfile.maintenance

```dockerfile
# ビルドステージ
FROM rust:1.84-slim AS builder

WORKDIR /app

# 依存関係のキャッシュ用
COPY Cargo.toml Cargo.lock ./
COPY migration ./migration
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --bin cleanup

# 実際のソースコードをコピーしてビルド
COPY src ./src
RUN cargo build --release --bin cleanup

# 実行ステージ
FROM debian:bookworm-slim

# 必要なパッケージをインストール
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# ビルド成果物をコピー
COPY --from=builder /app/target/release/cleanup /app/cleanup

# 実行
ENTRYPOINT ["/app/cleanup"]
```

### .env.maintenance

```bash
# DB接続情報（.env.appと同じ）
DATABASE_URL=postgresql://user:password@db:5432/dbname

# クリーンアップ設定
CLEANUP_RETENTION_DAYS=30

# ログレベル
RUST_LOG=info
```

### Cargo.tomlへのバイナリ追加

```toml
[[bin]]
name = "cleanup"
path = "src/bin/cleanup.rs"
```

## トランザクション管理

### 原則
- すべての削除処理は1つのトランザクション内で実行
- エラー時は自動的にロールバックされ、データの一貫性を保証
- トランザクション完了後、プロセスは終了（コンテナも停止）

### トランザクションスコープ

```rust
// src/bin/cleanup.rs
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cleanup_service = DataCleanupService::new(db);

    // execute()内でトランザクション管理
    match cleanup_service.execute().await {
        Ok(stats) => {
            info!("データクリーンアップが正常に完了しました");
            Ok(())
        }
        Err(e) => {
            error!(error = %e, "データクリーンアップに失敗しました");
            Err(e.into())
        }
    }
}

// DataCleanupService
impl DataCleanupService {
    pub async fn execute(&self) -> Result<CleanupStatistics> {
        let cleanup_before = Utc::now() - Duration::days(self.retention_days);

        // トランザクション開始
        let txn = self.db.begin().await?;

        // すべての削除処理が同一トランザクション内
        let deleted_recruitments = self.cleanup_battle_recruitments(&txn, cleanup_before).await?;
        let deleted_notifications = self.cleanup_notifications(&txn, cleanup_before).await?;
        let deleted_tasks = self.cleanup_scheduled_tasks(&txn, cleanup_before).await?;

        // コミット（エラー時は自動ロールバック）
        txn.commit().await?;

        Ok(CleanupStatistics {
            deleted_recruitments,
            deleted_notifications,
            deleted_tasks,
            cleanup_before,
        })
    }
}
```

## エラーハンドリング

### エラー種別と対応

| エラー種別 | 発生箇所 | 対応 |
|----------|---------|------|
| **DB接続エラー** | Database::connect() | エラーログ出力、プロセス終了（exit code: 1）、翌日cron再実行 |
| **トランザクション開始エラー** | db.begin() | エラーログ出力、プロセス終了（exit code: 1）、翌日cron再実行 |
| **削除クエリエラー** | 各テーブル削除時 | トランザクションロールバック、エラーログ出力、プロセス終了（exit code: 1）、翌日cron再実行 |
| **制約違反エラー** | 外部キー制約違反 | トランザクションロールバック、エラーログ出力、要調査（データ不整合の可能性） |
| **コミットエラー** | txn.commit() | エラーログ出力、プロセス終了（exit code: 1）、翌日cron再実行 |

### 終了コード

| 終了コード | 意味 | 対応 |
|----------|------|------|
| **0** | 正常終了 | 削除処理が成功、ログに統計情報を出力 |
| **1** | エラー終了 | 削除処理が失敗、エラーログを確認、翌日自動リトライ |

### ログ出力

```rust
impl DataCleanupService {
    pub async fn execute(&self) -> Result<CleanupStatistics> {
        info!("データクリーンアップを開始します");

        let cleanup_before = Utc::now() - Duration::days(self.retention_days);
        info!(cleanup_before = %cleanup_before, "削除基準日時を計算しました");

        let txn = self.db.begin().await?;

        let deleted_recruitments = self.cleanup_battle_recruitments(&txn, cleanup_before).await?;
        info!(deleted_count = deleted_recruitments, "battle_recruitmentsを削除しました");

        let deleted_notifications = self.cleanup_notifications(&txn, cleanup_before).await?;
        info!(deleted_count = deleted_notifications, "notificationsを削除しました");

        let deleted_tasks = self.cleanup_scheduled_tasks(&txn, cleanup_before).await?;
        info!(deleted_count = deleted_tasks, "scheduled_tasksを削除しました");

        txn.commit().await?;

        info!("データクリーンアップが正常に完了しました");

        Ok(CleanupStatistics {
            deleted_recruitments,
            deleted_notifications,
            deleted_tasks,
            cleanup_before,
        })
    }
}
```

### ログ出力例（正常時）

```
[INFO] データクリーンアップを開始します
[INFO] cleanup_before=2025-11-26T18:00:00Z 削除基準日時を計算しました
[INFO] deleted_count=2345 battle_recruitmentsを削除しました
[INFO] deleted_count=1234 notificationsを削除しました
[INFO] deleted_count=3456 scheduled_tasksを削除しました
[INFO] recruitments=2345 notifications=1234 tasks=3456 データクリーンアップが正常に完了しました
```

### ログ出力例（エラー時）

```
[INFO] データクリーンアップを開始します
[INFO] cleanup_before=2025-11-26T18:00:00Z 削除基準日時を計算しました
[INFO] deleted_count=2345 battle_recruitmentsを削除しました
[ERROR] error="database connection error: connection timeout" データクリーンアップに失敗しました
```

## パフォーマンス考慮事項

### 削除クエリの最適化

#### 1. インデックス活用
```sql
-- battle_recruitments の削除で使用されるインデックス
CREATE INDEX idx_battle_recruitments_cleanup
    ON worker.battle_recruitments(quest_start_at, is_recruiting);

-- notifications の削除で使用されるインデックス
CREATE INDEX idx_notifications_cleanup
    ON worker.notifications(schedule_datetime, is_sent);

-- scheduled_tasks の削除で使用されるインデックス
CREATE INDEX idx_scheduled_tasks_cleanup
    ON worker.scheduled_tasks(schedule_datetime, is_executed, task_type);
```

#### 2. バッチ削除の回避
- 30日分のデータを一度に削除するため、通常は数百〜数千レコード程度
- PostgreSQLの通常のDELETEクエリで十分に高速
- バッチ削除（LIMIT付きループ）は不要

#### 3. 実行時間の見積もり
- **想定データ量**: 1日あたり数十〜数百レコード
- **30日分**: 数千〜数万レコード
- **削除時間**: 数秒〜数十秒程度（インデックスあり）
- **実行時刻**: AM3時（ユーザー少ない時間帯）

### ロック競合の回避

```rust
// トランザクション分離レベル: READ COMMITTED (デフォルト)
// - 他のトランザクションとの競合を最小化
// - 削除対象は30日以上前のデータのため、通常は競合しない
// - AM3時実行のため、Bot本体の処理ピークタイムと重ならない

impl DataCleanupService {
    async fn cleanup_battle_recruitments(
        &self,
        txn: &DatabaseTransaction,
        cleanup_before: DateTime<Utc>,
    ) -> Result<u64> {
        // 短いクエリで高速実行、ロック時間を最小化
        let result = BattleRecruitments::delete_many()
            .filter(battle_recruitments::Column::QuestStartAt.lt(cleanup_before))
            .filter(battle_recruitments::Column::IsRecruiting.eq(false))
            .exec(txn)
            .await?;

        Ok(result.rows_affected)
    }
}
```

### Bot本体への影響

- **プロセス独立**: Maintenanceコンテナは別プロセスのため、Bot本体のメモリやCPUには影響しない
- **DB接続**: 別コネクションプールを使用するため、Botの接続数を圧迫しない
- **実行タイミング**: AM3時実行のため、ユーザー操作のピークタイム（夜間）を避ける
- **障害の影響範囲**: Maintenanceコンテナでエラーが発生しても、Bot本体は影響を受けない

## 監視とメンテナンス

### 監視項目

| 監視項目 | 確認方法 | 正常値 | 異常時対応 |
|---------|---------|-------|----------|
| **実行状況** | ログ出力確認 | 毎日AM3時に実行 | タスク再作成、Bot再起動 |
| **削除件数** | ログ出力確認 | 数千〜数万レコード/日 | データ量異常の調査 |
| **実行時間** | ログ出力確認 | 数秒〜数十秒 | インデックス確認、DB最適化 |
| **エラー発生** | エラーログ確認 | 0件 | エラー原因調査、修正 |
| **DB容量** | PostgreSQL監視 | 増加傾向なし | クリーンアップ設定見直し |

### ログ出力例

```
[INFO] task_id=12345 データクリーンアップを開始します
[DEBUG] task_id=12345 cleanup_before=2025-11-26T18:00:00Z 削除基準日時を取得しました
[INFO] task_id=12345 deleted_count=2345 battle_recruitmentsを削除しました
[INFO] task_id=12345 deleted_count=1234 notificationsを削除しました
[INFO] task_id=12345 deleted_count=3456 scheduled_tasksを削除しました
[INFO] task_id=12345 次回タスクを作成しました
[INFO] task_id=12345 データクリーンアップを完了しました
```

### メンテナンス手順

#### 1. 削除基準日数の変更

.env.maintenanceファイルを編集:

```bash
# 60日間保持に変更
CLEANUP_RETENTION_DAYS=60
```

または、docker compose実行時に指定:

```bash
docker compose run --rm -e CLEANUP_RETENTION_DAYS=60 maintenance
```

#### 2. 削除対象テーブルの追加

src/services/maintenance/data_cleanup_service.rsを編集:

```rust
impl DataCleanupService {
    pub async fn execute(&self) -> Result<CleanupStatistics> {
        // 既存の削除処理
        let deleted_recruitments = self.cleanup_battle_recruitments(&txn, cleanup_before).await?;
        let deleted_notifications = self.cleanup_notifications(&txn, cleanup_before).await?;
        let deleted_tasks = self.cleanup_scheduled_tasks(&txn, cleanup_before).await?;

        // 新しいテーブルの削除処理を追加
        let deleted_new_data = self.cleanup_new_table(&txn, cleanup_before).await?;

        txn.commit().await?;

        Ok(CleanupStatistics {
            deleted_recruitments,
            deleted_notifications,
            deleted_tasks,
            deleted_new_data,  // 統計情報に追加
            cleanup_before,
        })
    }

    async fn cleanup_new_table(
        &self,
        txn: &DatabaseTransaction,
        cleanup_before: DateTime<Utc>,
    ) -> Result<u64> {
        let result = NewTable::delete_many()
            .filter(new_table::Column::CreatedAt.lt(cleanup_before))
            .exec(txn)
            .await?;

        info!(deleted_count = result.rows_affected, "new_tableを削除しました");
        Ok(result.rows_affected)
    }
}
```

#### 3. 手動実行（緊急時・テスト）

```bash
# 通常実行
docker compose run --rm maintenance

# ドライランモード（削除せずログ出力のみ）
docker compose run --rm -e DRY_RUN=true maintenance

# 保持期間を変更して実行
docker compose run --rm -e CLEANUP_RETENTION_DAYS=90 maintenance

# 特定日時より前のデータを削除
docker compose run --rm -e CLEANUP_BEFORE="2025-01-01T00:00:00Z" maintenance
```

#### 4. cron設定の確認

```bash
# ホストOSのcron設定を確認
crontab -l

# cronログを確認
tail -f /var/log/cleanup.log

# docker compose logsで確認（cronコンテナ使用時）
docker compose logs -f cron
```

## セキュリティ考慮事項

### 1. 削除の不可逆性
- 削除されたデータは復元できない
- バックアップからの復元が必要になる場合がある
- 削除前のバックアップ取得を推奨

### 2. CASCADE削除の影響範囲
- 外部キー制約により、関連データも自動削除される
- 削除前に影響範囲を把握することが重要
- 本設計書の「削除対象テーブルとデータ」を参照

### 3. データ保護
- 誤削除を防ぐため、削除条件は厳格に設定
- `is_recruiting = false`, `is_sent = true`, `is_executed = true` などのフラグで保護
- 30日間の猶予期間により、誤削除のリスクを低減

## 拡張性

### 将来的な拡張ポイント

#### 1. 削除基準の柔軟化
```rust
// テーブルごとに異なる保持期間を設定
pub struct CleanupConfig {
    pub battle_recruitments_retention_days: i64,  // 30日
    pub notifications_retention_days: i64,        // 30日
    pub scheduled_tasks_retention_days: i64,      // 90日（長期保持）
}
```

#### 2. アーカイブ機能
```rust
// 削除前に別テーブルへアーカイブ
async fn archive_before_delete(
    &self,
    txn: &DatabaseTransaction,
    cleanup_before: DateTime<Utc>,
) -> Result<()> {
    // battle_recruitments → battle_recruitments_archive へコピー
    // その後削除
}
```

#### 3. 統計情報の保持
```rust
// 削除前に統計情報を集計・保存
async fn save_statistics_before_delete(
    &self,
    txn: &DatabaseTransaction,
    cleanup_before: DateTime<Utc>,
) -> Result<()> {
    // 削除対象データの統計（件数、傾向など）を保存
}
```

#### 4. 条件付き削除
```rust
// 特定条件のデータのみ削除
async fn cleanup_with_conditions(
    &self,
    txn: &DatabaseTransaction,
    cleanup_before: DateTime<Utc>,
) -> Result<u64> {
    // 例: キャンセルされた募集のみ削除
    BattleRecruitments::delete_many()
        .filter(battle_recruitments::Column::QuestStartAt.lt(cleanup_before))
        .filter(battle_recruitments::Column::IsCanceled.eq(true))
        .exec(txn)
        .await?
}
```

## 関連ドキュメント

- [データベースロール設計書](./data_cleanup_system_database_role.md) - Cleanupロールの詳細設計
- [スケジュール処理システム設計書](./schedule_processing_system.md) - 親システムの設計
- [データベース設計](../../database/README.md) - テーブル設計とリレーション
- [データベースロール使用ガイド](../../database/db_role_usage.md) - ロール全体の設計
- [Clean Architecture ガイドライン](../../../CLAUDE.md) - アーキテクチャ原則

## CI/CDとデプロイ

### GitHub Actionsでのビルド

`.github/workflows/build-maintenance.yml`:

```yaml
name: Build Maintenance Container

on:
  push:
    branches: [main]
    paths:
      - 'src/bin/cleanup.rs'
      - 'src/services/maintenance/**'
      - 'Dockerfile.maintenance'
      - 'Cargo.toml'

jobs:
  build:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      packages: write

    steps:
      - uses: actions/checkout@v4

      - name: Log in to GitHub Container Registry
        uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Build and push
        uses: docker/build-push-action@v5
        with:
          context: .
          file: Dockerfile.maintenance
          push: true
          tags: |
            ghcr.io/${{ github.repository }}-maintenance:latest
            ghcr.io/${{ github.repository }}-maintenance:${{ github.sha }}
```

### デプロイ手順

```bash
# 1. 最新イメージをpull
docker compose pull maintenance

# 2. テスト実行（手動実行で動作確認）
docker compose run --rm maintenance

# 3. cron設定を確認
crontab -l

# 4. ログ監視
tail -f /var/log/cleanup.log
```

## まとめ

データクリーンアップシステムは、以下の特徴を持つ:

1. **Bot本体からの独立**: 専用Maintenanceコンテナとして実装し、負荷を完全に分離
2. **既存コードベースの再利用**: Rustで実装し、Repository層、エンティティ、エラーハンドリング機構を再利用
3. **cron定期実行**: 毎日AM3時にcron起動し、シンプルかつ確実に実行
4. **安全性**: トランザクション管理とエラーハンドリングにより、データの一貫性を保証
5. **パフォーマンス**: インデックス最適化と深夜実行により、サービスへの影響を最小化
6. **拡張性**: 削除対象テーブルの追加や削除条件の変更が容易
7. **保守性**: 環境変数で設定可能、手動実行・テスト実行が容易

毎日深夜3時にcronで自動実行され、30日以上前の不要なデータを削除することで、DB容量を適切に管理し、システムの安定稼働を支える。

## 実装ファイル一覧

- `src/bin/cleanup.rs` - メインバイナリ
- `src/services/maintenance/data_cleanup_service.rs` - クリーンアップサービス
- `Dockerfile.maintenance` - Dockerイメージビルド定義
- `.env.maintenance` - 環境変数設定
- `docker-compose.yml` - Maintenanceコンテナ定義追加
- `.github/workflows/build-maintenance.yml` - CI/CDパイプライン
