# スケジュール処理システム設計書

## 概要

通知、解散、データ整理などの様々なスケジュール処理を統一的に管理するシステムの設計書。
既存の通知処理を踏襲しつつ、より汎用的で拡張性の高い基盤を構築する。

## 設計原則

### 1. パフォーマンスとコスト重視
- JSONBではなく専用テーブルでインデックスを活用
- 部分インデックスによる検索最適化
- プリロード戦略によるDB負荷の分散

### 2. 整合性優先
- 実行時のDB再確認により、タスク変更・削除に対する整合性を保証
- キャッシュ無効化などの複雑な仕組みは不要

### 3. 既存資産の活用
- `notifications`テーブルは変更せず、そのまま通知処理に使用
- 既存のトランザクション管理とエラーハンドリングを維持

### 4. 拡張性
- 新しいタスク種別を追加する際は、専用テーブルを追加するだけ
- タスク種別ごとに明確なデータ構造を持つ

## アーキテクチャ

### レイヤー構成

```
Events (schedule_handler.rs)
  ↓
Facade (SchedulerFacade)
  ↓ トランザクション境界
Services
  ├─ SchedulerManager (tokio-cron-scheduler統合)
  ├─ NotificationService (既存)
  ├─ DissolutionTaskExecutor (新規)
  ├─ RecurringRecruitmentTaskExecutor (新規)
  └─ DataCleanupTaskExecutor (新規)
  ↓
Repository
  ├─ NotificationRepository
  ├─ ScheduledTaskRepository
  ├─ ScheduledTaskNotificationRepository
  ├─ ScheduledTaskDissolutionRepository
  ├─ ScheduledTaskRecurringRecruitmentRepository
  ├─ ScheduledTaskCleanupRepository
  └─ BattleRecruitmentScheduleRepository
```

### コンポーネント設計

#### SchedulerManager

**責務:**
- tokio-cron-schedulerの初期化と管理
- プリロード処理の実行（10秒間隔）
- タスク実行時のDB再確認ロジック
- Bot起動時の過去タスク処理

**主要メソッド:**
```rust
pub struct SchedulerManager {
    scheduler: JobScheduler,
    app_state: Arc<AppState>,
    http: Arc<Http>,
}

impl SchedulerManager {
    /// 初期化処理
    /// - 過去の未実行タスクを実行
    /// - プリロードジョブを登録
    pub async fn initialize(&self) -> Result<()>;

    /// プリロード処理
    /// now ~ now+20秒の範囲でタスクを取得し、ジョブ登録
    async fn preload_tasks(&self) -> Result<()>;

    /// タスク実行（DB再確認あり）
    async fn execute_task(&self, task_id: i32, task_type: ScheduledTaskType) -> Result<()>;

    /// 通知実行（DB再確認あり）
    async fn execute_notification(&self, notification_id: i32) -> Result<()>;
}
```

#### DissolutionTaskExecutor

**責務:**
- 解散タスクの実行
- 参加者数のチェック
- 不足時の募集キャンセルとメッセージ送信

**主要メソッド:**
```rust
pub struct DissolutionTaskExecutor {
    recruitment_repo: RecruitmentRepository,
    participants_repo: ParticipantsRepository,
    message_service: MessageService,
    http: Arc<Http>,
}

impl DissolutionTaskExecutor {
    /// 解散タスクを実行
    /// - 参加者数をチェック
    /// - 不足していればキャンセル+メッセージ送信
    /// - 達成していれば何もしない
    pub async fn execute(&self, txn: &DatabaseTransaction, task_id: i32) -> Result<()>;
}
```

#### DataCleanupTaskExecutor

**責務:**
- 古いデータの削除
- 次回実行タスクの作成

**主要メソッド:**
```rust
pub struct DataCleanupTaskExecutor {
    cleanup_repo: CleanupRepository,
}

impl DataCleanupTaskExecutor {
    /// データ整理タスクを実行
    /// - 指定日時より前のデータを削除
    /// - 次回実行タスクを作成
    pub async fn execute(&self, txn: &DatabaseTransaction, task_id: i32) -> Result<()>;
}
```

## データモデル

### scheduled_tasksテーブル

すべてのスケジュール処理の基底テーブル（通知を含む）

```sql
CREATE TABLE worker.scheduled_tasks (
    id SERIAL PRIMARY KEY,
    schedule_datetime TIMESTAMPTZ NOT NULL,  -- 実行日時
    task_type INT NOT NULL,                  -- タスク種別（1:通知, 2:解散, 3:データ整理）
    guild_id BIGINT,                         -- ギルドID（NULL可、全体処理の場合）
    channel_id BIGINT,                       -- チャンネルID（NULL可）
    is_executed BOOLEAN NOT NULL DEFAULT FALSE,  -- 実行済みフラグ
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 部分インデックス: 未実行タスクに特化
CREATE INDEX idx_scheduled_tasks_datetime_not_executed
    ON worker.scheduled_tasks(schedule_datetime)
    WHERE is_executed = false;

CREATE INDEX idx_scheduled_tasks_type ON worker.scheduled_tasks(task_type);
CREATE INDEX idx_scheduled_tasks_guild ON worker.scheduled_tasks(guild_id);
```

### タスク種別の定義

```rust
pub enum ScheduledTaskType {
    Notification = 1,         // 通知
    Dissolution = 2,          // 解散
    DataCleanup = 3,          // データ整理
    RecurringRecruitment = 4, // 定期募集
}
```

**注:**
- 通知（task_type=1）はscheduled_tasksテーブルをベースとし、scheduled_task_notificationsテーブル経由でnotificationsテーブルと紐付けられます。
- 定期募集（task_type=4）はscheduled_tasksテーブルをベースとし、scheduled_task_recurring_recruitmentsテーブル経由でbattle_recruitment_schedulesテーブルと紐付けられます。

### 関連テーブル

#### 通知タスク（scheduled_task_notifications）

```sql
CREATE TABLE worker.scheduled_task_notifications (
    task_id INT NOT NULL REFERENCES worker.scheduled_tasks(id) ON DELETE CASCADE,
    notification_id INT NOT NULL REFERENCES worker.notifications(id) ON DELETE CASCADE,
    PRIMARY KEY (task_id)
);

CREATE INDEX idx_scheduled_task_notifications_notification
    ON worker.scheduled_task_notifications(notification_id);
```

**用途:**
- scheduled_tasks（task_type=1）とnotificationsテーブルの紐付け
- scheduled_tasksを母テーブルとして、通知も統一的なスケジュール処理で管理

#### 解散タスク（scheduled_task_dissolutions）

```sql
CREATE TABLE worker.scheduled_task_dissolutions (
    task_id INT NOT NULL REFERENCES worker.scheduled_tasks(id) ON DELETE CASCADE,
    recruit_id INT NOT NULL REFERENCES worker.battle_recruitments(id) ON DELETE CASCADE,
    PRIMARY KEY (task_id)
);

CREATE INDEX idx_scheduled_task_dissolutions_recruit
    ON worker.scheduled_task_dissolutions(recruit_id);
```

**用途:**
- マルチ募集の解散処理
- 指定時刻に参加者数をチェックし、不足していれば自動キャンセル

#### データ整理タスク（scheduled_task_cleanups）

```sql
CREATE TABLE worker.scheduled_task_cleanups (
    task_id INT NOT NULL REFERENCES worker.scheduled_tasks(id) ON DELETE CASCADE,
    target_schema VARCHAR NOT NULL,      -- 対象スキーマ（worker, guild_masterなど）
    target_table VARCHAR NOT NULL,       -- 対象テーブル名
    cleanup_before TIMESTAMPTZ NOT NULL, -- この日時より前のデータを削除
    PRIMARY KEY (task_id)
);
```

**用途:**
- 定期的な古いデータの削除（AM3〜5時）
- `battle_recruitments`、`notifications`など

#### 定期募集タスク（scheduled_task_recurring_recruitments）

```sql
CREATE TABLE worker.scheduled_task_recurring_recruitments (
    task_id INT NOT NULL REFERENCES worker.scheduled_tasks(id) ON DELETE CASCADE,
    schedule_id INT NOT NULL REFERENCES worker.battle_recruitment_schedules(id) ON DELETE CASCADE,
    PRIMARY KEY (task_id)
);

CREATE INDEX idx_scheduled_task_recurring_recruitments_schedule
    ON worker.scheduled_task_recurring_recruitments(schedule_id);
```

**用途:**
- 定期募集スケジュールからのマルチ募集自動作成
- 次回実行タスクを自動的に作成（過去日時の場合は未来日時が見つかるまで繰り返し計算）
- スケジュールが無効化されている場合は実行をスキップ

### 既存テーブルとの関係

#### notifications（既存・変更なし）

```sql
-- 既存テーブル（変更なし）
CREATE TABLE worker.notifications (
    id SERIAL PRIMARY KEY,
    schedule_datetime TIMESTAMPTZ NOT NULL,
    guild_id BIGINT NOT NULL,
    channel_id BIGINT NOT NULL,
    message_text_id VARCHAR NOT NULL,
    is_sent BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**役割:** 既存の通知処理をそのまま継続。

## 処理フロー

### Bot起動時の初期化

```
SchedulerManager::initialize()
  ↓
  ├─ tokio-cron-schedulerの初期化（メモリベース）
  │
  ├─ [過去の未実行タスク処理] Bot停止中に実行されるべきだったタスクを実行
  │   ├─ last_process_times.execute_time取得
  │   ├─ execute_time ~ now の範囲で未実行タスクを取得
  │   │   - notifications (is_sent=false)
  │   │   - scheduled_tasks (is_executed=false)
  │   └─ 各タスクを順次実行（実行前にDB再確認）
  │
  ├─ last_process_timesを現在時刻に更新
  │
  └─ [プリロードジョブ登録] 10秒間隔でプリロード処理を実行
      └─ "*/10 * * * * *" (10秒ごと)
```

### プリロード処理（10秒間隔）

```
SchedulerManager::preload_and_execute_tasks()
  ↓
  ├─ 未実行タスクを取得（過去〜now+20秒）
  │   └─ scheduled_tasks (is_executed=false、過去の未実行タスクも取得)
  │
  └─ 実行時刻に達しているタスクを即座に実行
      ├─ schedule_datetime <= now のタスクを実行
      └─ task_typeに応じた処理を実行（通知、解散、データ整理）
```

**注:** プリロード後にtokio-cron-schedulerへの個別登録は行わず、次回のプリロードサイクル（10秒後）で再度チェックします。実行時刻に達したタスクは即座に実行されます。

### タスク実行時（schedule_datetimeに到達）

```
SchedulerManager::preload_and_execute_tasks()
  ↓
  各タスクについて:
    ├─ schedule_datetime <= now か確認
    │   └─ まだ時刻に達していない → スキップ（次回サイクルで再チェック）
    │
    └─ task_typeに応じた処理を実行
        │
        ├─ [task_type = 1: 通知処理]
        │   ├─ scheduled_task_notificationsから notification_id を取得
        │   ├─ notificationsテーブルから通知情報を取得
        │   ├─ NotificationService::send_single_notification() を実行
        │   └─ scheduled_tasks.is_executed = true に更新
        │
        ├─ [task_type = 2: 解散処理]
        │   ├─ DissolutionTaskExecutor::execute() を実行
        │   │   ├─ 実行時DB再確認（タスク削除/実行済みチェック）
        │   │   ├─ 募集情報を取得
        │   │   ├─ Discordメッセージを更新
        │   │   ├─ DBで募集をキャンセル
        │   │   └─ 通知メッセージを送信
        │   └─ scheduled_tasks.is_executed = true に更新
        │
        ├─ [task_type = 3: データ整理処理]
        │   └─ DataCleanupTaskExecutor::execute() (未実装)
        │
        └─ [task_type = 4: 定期募集処理]
            ├─ RecurringRecruitmentTaskExecutor::execute() を実行
            │   ├─ 実行時DB再確認（タスク削除/実行済みチェック）
            │   ├─ スケジュール情報を取得（battle_recruitment_schedules）
            │   ├─ スケジュールが無効化されていないか確認
            │   ├─ RecruitmentCreationService でマルチ募集を作成
            │   ├─ 次回実行日時を計算（過去日時なら未来まで繰り返し計算）
            │   └─ 次回のscheduled_taskを作成
            └─ scheduled_tasks.is_executed = true に更新
```

### CRUD操作との関係

#### タスク作成時

```rust
// イベント通知スケジュール作成時（SchedulerService::save_calculated_schedules）
async fn save_calculated_schedules(&self, schedules: Vec<CalculatedSchedule>) -> Result<()> {
    let txn = self.db.begin().await?;

    for schedule in schedules {
        // 1. notificationを作成
        let notification = notification_repo.create_with_txn(&txn, ...).await?;

        // 2. notification_relを作成
        rel_repo.create_with_txn(&txn, ...).await?;

        // 3. scheduled_taskを作成（task_type=1: Notification）
        let scheduled_task = scheduled_task_repo.create(
            &txn,
            schedule.schedule_datetime,
            ScheduledTaskType::Notification.as_i32(),
            Some(schedule.guild_id),
            Some(schedule.channel_id)
        ).await?;

        // 4. scheduled_task_notificationを作成（紐付け）
        scheduled_task_notification_repo.create(
            &txn,
            scheduled_task.id,
            notification.id
        ).await?;
    }

    txn.commit().await?;

    // ✅ SchedulerManagerへの登録は不要
    // → プリロード処理（10秒間隔）が自動的に検出して実行する
}

// マルチ募集作成時
async fn create_recruitment(&self, params: RecruitmentParams) -> Result<()> {
    let txn = self.db.begin().await?;

    // 募集を作成
    let recruitment = self.recruitment_repo.create(&txn, params).await?;

    // 解散タスクを作成
    for dissolution_time in params.dissolution_times {
        let task = self.scheduled_task_repo.create(
            &txn,
            dissolution_time,
            ScheduledTaskType::Dissolution.as_i32(),
            Some(recruitment.guild_id),
            Some(recruitment.channel_id),
        ).await?;

        self.dissolution_repo.create(&txn, task.id, recruitment.id).await?;
    }

    txn.commit().await?;

    // ✅ SchedulerManagerへの登録は不要
    // → プリロード処理（10秒間隔）が自動的に検出して実行する
}

// 定期募集スケジュール作成時（ScheduleCreateService::create_schedule）
async fn create_schedule(&self, txn: &DatabaseTransaction, params: ScheduleParams) -> Result<()> {
    // 1. スケジュールを作成
    let (schedule, days) = schedule_repo.create_with_txn(&txn, ...).await?;

    // 2. 次回実行日時を計算してscheduled_tasksに登録
    let now = Utc::now();
    let mut search_from = now;

    // 未来の次回実行日時が見つかるまでループ（最大365日先まで）
    loop {
        let search_to = search_from + Duration::days(7);
        let next_times = self.schedule_service
            .calculate_next_recruitment_times(&schedule, &days, search_from, search_to)?;

        if let Some(next_time) = next_times.first() {
            if next_time.recruit_start_at > now {
                // scheduled_taskを作成（task_type=4: RecurringRecruitment）
                let task = scheduled_task_repo.create(
                    &txn,
                    next_time.recruit_start_at,
                    ScheduledTaskType::RecurringRecruitment.as_i32(),
                    Some(next_time.guild_id),
                    Some(next_time.channel_id),
                ).await?;

                // scheduled_task_recurring_recruitmentsに紐付け
                recurring_repo.create(&txn, task.id, schedule.id).await?;

                break;
            }
        }

        search_from = search_to;
        // 無限ループ防止
        if (search_from - now).num_days() > 365 {
            return Err(AppError::Business { message: "次回実行日時が見つかりません" });
        }
    }

    // ✅ SchedulerManagerへの登録は不要
    // → プリロード処理（10秒間隔）が自動的に検出して実行する
}
```

#### タスク更新・削除時

```rust
// 通知削除時（NotificationManagementService::delete_recruitment_notifications）
async fn delete_recruitment_notifications(&self, recruit_id: i32) -> Result<()> {
    let txn = self.db.begin().await?;

    // notification_idリストを取得
    let notification_ids = notification_rel_repo
        .find_by_recruit_id_with_txn(&txn, recruit_id).await?
        .into_iter()
        .map(|rel| rel.notification_id)
        .collect::<Vec<_>>();

    for notification_id in notification_ids {
        // scheduled_task_notificationからtask_idを取得
        if let Some(task_rel) = scheduled_task_notification_repo
            .find_by_notification_id(&txn, notification_id).await?
        {
            // scheduled_taskを削除（CASCADEでscheduled_task_notificationも削除される）
            scheduled_task_repo.delete_by_id(&txn, task_rel.task_id).await?;
        }

        // notification_relとnotificationを削除
        notification_rel_repo.delete_by_notification_id_with_txn(&txn, notification_id).await?;
        notification_repo.delete_by_id_with_txn(&txn, notification_id).await?;
    }

    txn.commit().await?;

    // ✅ SchedulerManagerのジョブキャンセルは不要
    // → 削除されたタスクは次回プリロード時に取得されない
}

// マルチ募集の解散時刻変更時
async fn update_recruitment_dissolution(&self, recruit_id: i32, new_times: Vec<DateTime<Utc>>) -> Result<()> {
    let txn = self.db.begin().await?;

    // 既存の解散タスクを削除（CASCADE）
    self.scheduled_task_repo
        .delete_dissolutions_by_recruit_id(&txn, recruit_id).await?;

    // 新しいタスクを作成
    for time in new_times {
        let task = self.scheduled_task_repo.create(
            &txn,
            time,
            ScheduledTaskType::Dissolution.as_i32(),
            ...,
        ).await?;

        self.dissolution_repo.create(&txn, task.id, recruit_id).await?;
    }

    txn.commit().await?;

    // ✅ SchedulerManagerのジョブキャンセルは不要
    // → 削除されたタスクは次回プリロード時に取得されない
}
```

## 整合性保証の仕組み

### 問題: タスク変更・削除とプリロード済みジョブの不整合

**シナリオ:**
1. プリロード処理でタスクA（15:30実行）をtokio-cron-schedulerに登録
2. ユーザーがタスクAを削除
3. 15:30になり、削除済みのタスクAが実行されてしまう

### 解決策: 実行時DB再確認

タスク実行前に必ずDBから最新状態を取得し、有効性を確認する。

```rust
async fn execute_task(&self, task_id: i32, task_type: ScheduledTaskType) -> Result<()> {
    // DB再確認
    let task = match self.scheduled_task_service.get_task_by_id(task_id).await? {
        Some(t) if !t.is_executed => t,  // 有効なタスク
        _ => {
            debug!(task_id, "タスクは既に実行済みまたは削除されています");
            return Ok(());  // スキップ
        }
    };

    // タスク種別に応じた処理
    match task_type {
        ScheduledTaskType::Dissolution => {
            self.dissolution_executor.execute(task_id).await?;
        }
        ScheduledTaskType::DataCleanup => {
            self.data_cleanup_executor.execute(task_id).await?;
        }
    }

    Ok(())
}
```

**メリット:**
- ジョブIDの管理が不要
- UPDATE/DELETE時にtokio-cron-schedulerのジョブキャンセル不要
- Bot再起動でジョブIDが変わっても問題ない
- シンプルで保守性が高い

**デメリット:**
- 削除されたジョブも実行時刻まで待つ（ただし実行はスキップされるため実害なし）

## パフォーマンス最適化

### 1. 部分インデックス

未実行タスクのみをインデックス化し、検索を高速化。

```sql
CREATE INDEX idx_scheduled_tasks_datetime_not_executed
    ON worker.scheduled_tasks(schedule_datetime)
    WHERE is_executed = false;

CREATE INDEX idx_notifications_datetime_not_sent
    ON worker.notifications(schedule_datetime)
    WHERE is_sent = false;
```

### 2. プリロード戦略

10秒間隔で20秒先までのタスクを先読みし、メモリとDB負荷のバランスを取る。

- **DB負荷**: 8,640回/日（10秒 × 2テーブル）
- **メモリ**: 20秒分のタスクのみ保持（数十〜数百レコード程度）

### 3. JOIN最適化

Repositoryレイヤーで事前にJOINし、Serviceレイヤーでは複数クエリを発行しない。

```rust
// ❌ N+1問題
for task in tasks {
    let dissolution = self.find_dissolution(task.id).await?; // 毎回クエリ
}

// ✅ 事前JOIN
let tasks_with_dissolutions = self.repository
    .find_dissolutions_in_range(now, to).await?; // 1回のJOINクエリ
```

## トランザクション管理

### 原則

- **Facade層**: トランザクション境界を管理
- **Service層**: トランザクションを受け取り、ビジネスロジックを実行
- **Repository層**: トランザクション経由でDB操作

### 例: 解散タスク実行

```rust
// Facade層
pub async fn execute_all_schedules(&self) -> Result<()> {
    let txn = self.app_state.system_db().begin().await?;

    let result = self.scheduler_manager.execute_pending_tasks(&txn).await;

    match result {
        Ok(_) => {
            txn.commit().await?;
            Ok(())
        }
        Err(e) => {
            txn.rollback().await?;
            Err(e)
        }
    }
}

// Service層
impl DissolutionTaskExecutor {
    pub async fn execute(&self, txn: &DatabaseTransaction, task_id: i32) -> Result<()> {
        // トランザクション経由でDB操作
        let dissolution = self.repo.find_by_task_id(txn, task_id).await?;

        // 参加者数チェック
        let participants = self.participants_repo.count(txn, dissolution.recruit_id).await?;

        if participants < required {
            // キャンセル処理
            self.recruitment_repo.cancel(txn, dissolution.recruit_id).await?;
        }

        // タスク完了
        self.scheduled_task_repo.mark_executed(txn, task_id).await?;

        Ok(())
    }
}
```

## エラーハンドリング

### 原則

- すべてのエラーは`AppError`型で統一
- Facade層でエラーをログ出力し、トランザクションをロールバック
- Service層ではエラーを`?`で伝播

### エラー種別

```rust
pub enum AppError {
    Database { message: String, source: DbErr },
    Business { message: String },
    External { message: String, source: Box<dyn std::error::Error> },
}
```

### 例: エラーハンドリング

```rust
// Facade層
pub async fn execute_dissolution(&self, task_id: i32) -> Result<()> {
    let txn = self.app_state.guild_db().begin().await?;

    let result = self.dissolution_executor.execute(&txn, task_id).await;

    match result {
        Ok(_) => {
            txn.commit().await?;
            info!(task_id, "解散タスクを実行しました");
            Ok(())
        }
        Err(e) => {
            error!(task_id, error = %e, "解散タスクの実行に失敗しました");
            txn.rollback().await?;
            Err(e)
        }
    }
}

// Service層
impl DissolutionTaskExecutor {
    pub async fn execute(&self, txn: &DatabaseTransaction, task_id: i32) -> Result<()> {
        let dissolution = self.repo.find_by_task_id(txn, task_id).await?; // エラー伝播

        let participants = self.participants_repo.count(txn, dissolution.recruit_id).await?;

        if participants < required {
            self.recruitment_repo.cancel(txn, dissolution.recruit_id).await?;
        }

        self.scheduled_task_repo.mark_executed(txn, task_id).await?;

        Ok(())
    }
}
```

## ログ出力

### 原則

- 構造化ログ（tracing）を使用
- Facade層とService層でログ出力
- Repository層では基本的にログ出力しない

### ログレベル

- **debug**: プリロード処理、タスク取得
- **info**: タスク実行開始・完了、重要な状態変化
- **error**: エラー発生時

### 例

```rust
// Facade層
info!("スケジュール処理を開始します");
debug!(task_count = tasks.len(), "タスクを取得しました");
error!(error = %e, "スケジュール処理に失敗しました");

// Service層
info!(task_id = task.id, "解散タスクを実行します");
debug!(recruit_id = dissolution.recruit_id, participants = count, "参加者数をチェックしました");
```

## 拡張性

### 新しいタスク種別の追加

1. **タスク種別の定義**
   ```rust
   pub enum ScheduledTaskType {
       Notification = 1,
       Dissolution = 2,
       DataCleanup = 3,
       RecurringRecruitment = 4,
       NewTaskType = 5,  // 追加
   }
   ```

2. **関連テーブルの作成**
   ```sql
   CREATE TABLE worker.scheduled_task_new_type (
       task_id INT NOT NULL REFERENCES worker.scheduled_tasks(id) ON DELETE CASCADE,
       -- 固有のカラム
       PRIMARY KEY (task_id)
   );
   ```

3. **Executorの実装**
   ```rust
   pub struct NewTypeExecutor {
       // 依存関係
   }

   impl NewTypeExecutor {
       pub async fn execute(&self, txn: &DatabaseTransaction, task_id: i32) -> Result<()> {
           // 実装
       }
   }
   ```

4. **SchedulerManagerへの統合**
   ```rust
   match task_type {
       ScheduledTaskType::Notification => {
           self.notification_service.send_single_notification(txn, notification_id).await?;
       }
       ScheduledTaskType::Dissolution => {
           self.dissolution_executor.execute(txn, task_id).await?;
       }
       ScheduledTaskType::DataCleanup => {
           self.data_cleanup_executor.execute(txn, task_id).await?;
       }
       ScheduledTaskType::RecurringRecruitment => {
           self.recurring_recruitment_executor.execute(txn, db, http, task_id).await?;
       }
       ScheduledTaskType::NewTaskType => {  // 追加
           self.new_type_executor.execute(txn, task_id).await?;
       }
   }
   ```

## 技術スタック

- **スケジューラー**: tokio-cron-scheduler（メモリベース、persistence機能は不使用）
- **ORM**: SeaORM
- **DB**: PostgreSQL
- **非同期ランタイム**: Tokio
- **ログ**: tracing + tracing-subscriber
- **エラーハンドリング**: thiserror

## まとめ

このスケジュール処理システムは、以下の特徴を持つ：

1. **パフォーマンス**: インデックス最適化とプリロード戦略により、DB負荷を最小化
2. **整合性**: 実行時DB再確認により、タスク変更・削除に確実に対応
3. **拡張性**: 新しいタスク種別を簡単に追加可能
4. **保守性**: Clean Architectureに準拠し、レイヤー分離が明確
5. **既存資産の活用**: 通知処理の仕組みを維持しつつ、汎用化

既存の通知処理の仕組みは維持しつつ、パフォーマンス、コスト、整合性を重視した汎用的なスケジュール処理基盤を提供する。
