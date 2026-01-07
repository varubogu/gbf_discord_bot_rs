# スケジュール処理システム 設計書

## 概要

tokio-cron-schedulerを使用してスケジュールタスクを定期的にプリロードし実行するシステムの設計書です。

## 実装したコンポーネント

### 1. データベーススキーマ

#### migration/src/m20251225_010000_create_scheduled_tasks.rs

4つのテーブルを作成:

- **worker.scheduled_tasks**: すべてのスケジュールタスクの基本情報
  - id, schedule_datetime, task_type, guild_id, channel_id, is_executed, created_at, updated_at
  - task_type: 1=通知, 2=解散, 3=データ整理
  - 部分インデックス: `WHERE is_executed = false` でパフォーマンス最適化

- **worker.scheduled_task_notifications**: 通知タスクの詳細情報
  - task_id (FK), notification_id (FK)
  - scheduled_tasks（task_type=1）とnotificationsテーブルを紐付ける

- **worker.scheduled_task_dissolutions**: 解散タスクの詳細情報
  - task_id (FK), recruit_id (FK)

- **worker.scheduled_task_cleanups**: データ整理タスクの詳細情報
  - task_id (FK), target_schema, target_table, cleanup_before

### 2. Entity層

#### src/models/entities/scheduled_tasks.rs

- `ScheduledTaskType` enum を追加:
  - `Notification = 1` (通知タスク)
  - `Dissolution = 2` (解散タスク)
  - `DataCleanup = 3` (データ整理タスク)

#### src/models/entities/scheduled_task_notifications.rs

- scheduled_task_notificationsテーブルのEntity定義
- task_idとnotification_idの紐付けを管理

### 3. Repository層

#### src/repository/database/schedule/scheduled_task_repository.rs

主要メソッド:
- `find_pending_in_range()`: 指定範囲内の未実行タスクを取得
- `find_pending_to()`: 指定日時以前の未実行タスクを取得（プリロード用、過去の未実行タスクも含む）
- `find_by_id()`: タスクIDで取得（実行時DB再確認用）
- `create()`: 新規タスク作成
- `mark_as_executed()`: タスクを実行済みにマーク
- `delete_by_id()`: タスクを削除
- `delete_dissolutions_by_recruit_id()`: 募集IDに紐づく解散タスクを削除

#### src/repository/database/schedule/scheduled_task_notification_repository.rs

主要メソッド:
- `find_pending_in_range()`: 未実行通知タスクをJOIN済みで取得
- `find_by_task_id()`: task_idで通知関連情報を取得
- `find_by_notification_id()`: notification_idで通知関連情報を取得
- `create()`: 通知タスク関連情報を作成
- `delete_by_notification_id()`: notification_idで通知タスクを削除

#### src/repository/database/schedule/scheduled_task_dissolution_repository.rs

主要メソッド:
- `find_pending_in_range()`: 未実行解散タスクをJOIN済みで取得
- `find_by_task_id()`: task_idで解散情報を取得
- `create()`: 解散タスク作成
- `find_by_recruit_id()`: recruit_idで解散タスクを取得

#### src/repository/database/schedule/scheduled_task_cleanup_repository.rs

将来のデータ整理機能用（現在は未使用）

### 4. Service層 - Executor

#### src/services/schedule/dissolution_task_executor.rs

**役割**: 解散タスクの実行ロジックを担当

**処理フロー**:
1. 実行時DB再確認（タスクが削除/実行済みでないかチェック）
2. 募集情報を取得
3. 既にキャンセル済みの場合はスキップ
4. Discordメッセージを取得
5. メッセージを更新（打ち消し線 + キャンセル済み表記）
6. DBで募集をキャンセル済み状態に更新
7. 参加者に通知メッセージを送信
8. タスクを実行済みにマーク

**実行結果**:
- `Cancelled`: 募集をキャンセルした
- `SkippedDueToSufficientParticipants`: 人数条件を満たしているためスキップ（※現在は未使用）
- `RecruitmentNotFound`: 募集が見つからない
- `AlreadyCancelled`: 既にキャンセル済み
- `DiscordMessageNotFound`: Discordメッセージが見つからない

### 5. Service層 - SchedulerManager

#### src/services/schedule/scheduler_manager.rs

**役割**: tokio-cron-schedulerを管理し、定期的にタスクをプリロード・実行

**主要機能**:

1. **初期化 (`new()`)**:
   - JobSchedulerを作成
   - 必要なリポジトリとサービスをDI

2. **起動 (`start()`)**:
   - 10秒間隔のcronジョブを登録 (`*/10 * * * * *`)
   - スケジューラーを起動

3. **プリロード処理 (`preload_and_execute_tasks()`)**:
   - 過去〜現在時刻+20秒の未実行タスクを取得（`find_pending_to`）
   - 実行時刻に達しているタスク（schedule_datetime <= now）を即座に実行
   - タスク種別（task_type）に応じて処理を分岐:
     - **task_type=1（通知）**: scheduled_task_notifications経由でnotificationsを取得し、NotificationServiceで送信
     - **task_type=2（解散）**: DissolutionTaskExecutorで募集を解散
     - **task_type=3（データ整理）**: 未実装

4. **停止 (`stop()`)**:
   - スケジューラーをシャットダウン

**処理パターン**:
```
10秒ごとのcronジョブ
  ↓
  過去〜now+20秒 の範囲で未実行タスクを取得（Bot停止中のタスクも含む）
  ↓
  各タスクについて:
    - schedule_datetime <= now ならば即座に実行
    - まだ実行時刻でない場合はスキップ（次回のプリロードで拾われる）
  ↓
  task_typeに応じた処理を実行
    - 1: NotificationServiceで通知送信 + is_executedを更新
    - 2: DissolutionTaskExecutorで解散処理 + is_executedを更新
    - 3: (未実装)
```

### 6. その他の変更

#### src/repository/battle_recruitments_repository.rs

- `get_by_id_with_txn()` メソッドを追加
  - DissolutionTaskExecutorから募集情報を取得するために必要

## アーキテクチャ設計の特徴

### 1. 実行時DB再確認パターン

タスクを実行する直前に必ずDBから最新状態を取得し、削除/実行済みでないかを確認します。これにより、以下の利点があります:

- キャッシュ無効化が不要
- CRUD操作とスケジューラーの疎結合
- データ整合性の保証

### 2. プリロード戦略

- **間隔**: 10秒
- **先読み範囲**: 20秒
- **利点**:
  - タスク実行の遅延を最小化
  - DBクエリの頻度と範囲のバランスが良い
  - 10秒以内に作成されたタスクも確実に実行される

### 3. PostgreSQL独立管理

- tokio-cron-schedulerのpersistence機能は使用しない
- 独自のテーブルでタスクを管理
- メモリベース（SimpleMetadataStore）で動作

### 4. トランザクション管理

- 各実行単位でトランザクションを開始・コミット
- エラー時のロールバックを保証
- Repository層での一貫したトランザクション対応

## 主要機能

1. **Bot起動時の過去タスク処理**
   - `find_pending_to`により過去の未実行タスクも自動的に処理
   - Bot停止中に実行できなかったタスクは起動後の最初のプリロードサイクルで実行

2. **通知処理との統合**
   - scheduled_tasks（task_type=1）+ scheduled_task_notificationsでnotificationsを管理
   - SchedulerService::save_calculated_schedulesでscheduled_tasks + scheduled_task_notificationsを作成
   - NotificationManagementServiceでも同様に作成・削除を実装
   - SchedulerManagerで通知を実行

3. **解散タスク処理**
   - DissolutionTaskExecutorによる解散処理
   - 募集キャンセル + Discordメッセージ更新 + 通知送信

## 拡張ポイント

1. **DataCleanupTaskExecutor**
   - データ整理タスクの実行ロジック
   - 古いレコードの削除処理

2. **Facade層での統合**
   - 募集作成時に解散タスクを自動作成
   - 募集削除時に解散タスクも削除

3. **テストコード**
   - モックを使った単体テスト
   - 統合テスト

## 使用方法（将来の統合イメージ）

### SchedulerManagerの起動

```rust
use crate::services::schedule::SchedulerManager;

// Bot起動時
let mut scheduler_manager = SchedulerManager::new(
    db,
    http,
    task_repo,
    dissolution_repo,
    recruitment_repo,
    participants_repo,
    message_service,
).await?;

scheduler_manager.start().await?;

// Bot終了時
scheduler_manager.stop().await?;
```

### 解散タスクの作成（Facade層での実装例）

```rust
// マルチ募集作成時
async fn create_recruitment_with_dissolution(
    &self,
    params: RecruitmentParams,
    dissolution_datetime: DateTime<Utc>,
) -> Result<()> {
    let txn = self.db.begin().await?;

    // 募集を作成
    let recruitment = self.recruitment_repo.create(&txn, params).await?;

    // scheduled_taskを作成
    let task = self.scheduled_task_repo.create(
        &txn,
        dissolution_datetime,
        ScheduledTaskType::Dissolution,
        Some(recruitment.guild_id),
        Some(recruitment.channel_id),
    ).await?;

    // 解散タスクの詳細情報を作成
    self.dissolution_repo.create(
        &txn,
        task.id,
        recruitment.id,
    ).await?;

    txn.commit().await?;

    // ✅ tokio-cron-schedulerへの登録は不要
    // プリロード処理（10秒間隔）が自動的に拾って実行する

    Ok(())
}
```

### 解散タスクの削除（募集キャンセル時）

```rust
async fn cancel_recruitment(&self, recruitment_id: i32) -> Result<()> {
    let txn = self.db.begin().await?;

    // 募集をキャンセル
    self.recruitment_repo.set_canceled(&txn, recruitment_id).await?;

    // 関連する解散タスクを削除
    self.scheduled_task_repo
        .delete_dissolutions_by_recruit_id(&txn, recruitment_id)
        .await?;

    txn.commit().await?;
    Ok(())
}
```

## パフォーマンス最適化

### 部分インデックス

```sql
CREATE INDEX idx_scheduled_tasks_datetime_not_executed
    ON worker.scheduled_tasks(schedule_datetime)
    WHERE is_executed = false;
```

未実行タスクのみをインデックス化することで:
- インデックスサイズを削減
- クエリパフォーマンスを向上

### プリロード範囲の最適化

- 20秒先読み: 十分な余裕を持ちつつ、DBクエリ範囲を最小化
- 10秒間隔: 新規作成タスクを迅速に検出

## 設計ドキュメント

詳細な設計については以下を参照:
- [docs/develop/design/features/schedule_processing_system.md](../design/features/schedule_processing_system.md)

## 次のステップ

1. Bot起動時の過去タスク処理を実装
2. Facade層での解散タスク作成・削除の統合
3. テストコードの作成
4. 定期募集機能との統合
5. DataCleanupTaskExecutorの実装
