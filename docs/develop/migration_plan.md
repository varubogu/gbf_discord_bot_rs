# SchedulerService → SchedulerManager 移行設計書

## 現状分析

### 既存のSchedulerService（廃止済み）

**役割:**
1. イベント通知スケジュールの生成（generate_schedules）
2. 通知の実行（execute_notifications） - NotificationServiceを呼び出す
3. 定期募集の実行（execute_recruitment_schedules）
4. last_process_timesの管理

**問題点:**
- 定期的にFacadeから呼び出す必要がある（外部cronや手動実行）
- 通知と定期募集が別々の処理フロー
- 解散タスクなどの新機能を追加しづらい

### 新しいSchedulerManager（実装済み）

**役割:**
1. tokio-cron-schedulerで10秒間隔のプリロード処理
2. scheduled_tasksテーブルのタスクを実行
3. 実行時DB再確認パターン

**現在の実装状況:**
- ✅ DissolutionTaskExecutor実装済み
- ❌ NotificationServiceとの統合は未実装
- ❌ 定期募集との統合は未実装

## 統合方針

### Phase 1: NotificationServiceとの統合（推奨）

既存の`notifications`テーブルも新しいSchedulerManagerで処理するように統合します。

**変更内容:**

1. **SchedulerManagerの拡張**
   - `preload_and_execute_tasks()`でnotificationsテーブルも取得
   - `is_sent = false`の通知をNotificationServiceに渡して実行
   - 実行後に`is_sent = true`に更新

2. **NotificationServiceの変更**
   - 既存の`execute_scheduled_notifications()`は維持（後方互換性）
   - 新しい`execute_single_notification()`メソッドを追加（1件ずつ実行用）

3. **SchedulerFacadeの変更**
   - `generate_schedules()`は維持（イベント通知スケジュール生成）
   - `execute_notifications()`は廃止 → SchedulerManagerが自動実行
   - `execute_recruitment_schedules()`は維持（定期募集は別途検討）

### Phase 2: 定期募集との統合（将来）

定期募集もscheduled_tasksで管理するように変更します（大規模な変更になるため、Phase 1完了後に検討）。

## 実装計画

### Step 1: NotificationServiceの拡張

```rust
// src/services/schedule/notification_service.rs

impl NotificationService {
    /// 単一の通知を実行（SchedulerManager用）
    pub async fn execute_single_notification(
        &self,
        txn: &DatabaseTransaction,
        notification_id: i32,
    ) -> Result<()> {
        // 通知情報を取得
        // Discordにメッセージを送信
        // is_sent = trueに更新
    }
}
```

### Step 2: SchedulerManagerの拡張

```rust
// src/services/schedule/scheduler_manager.rs

async fn preload_and_execute_tasks(...) -> Result<()> {
    // 既存: scheduled_tasksの取得
    let tasks = task_repo.find_pending_in_range(&txn, now, preload_until).await?;

    // 新規: notificationsの取得
    let notifications = notification_repo.find_pending_in_range(&txn, now, preload_until).await?;

    // scheduled_tasksの実行（既存のロジック）
    for task in tasks { ... }

    // notificationsの実行（新規）
    for notification in notifications {
        if notification.schedule_datetime <= now {
            notification_service.execute_single_notification(&txn, notification.id).await?;
        }
    }
}
```

### Step 3: SchedulerFacadeの段階的移行

**移行前（現状）:**
```rust
// Facadeから定期的に呼び出す
scheduler_facade.execute_notifications(http).await?;
```

**移行後:**
```rust
// SchedulerManagerが自動実行するため、Facade呼び出しは不要
// SchedulerManagerは起動時に1回startするだけ
scheduler_manager.start().await?;
```

### Step 4: 既存コードの廃止

**廃止するコード:**
- `SchedulerFacade::execute_notifications()` - SchedulerManagerが自動実行
- last_process_timesの通知関連の処理 - 不要になる（is_sentフラグで管理）

**維持するコード:**
- `SchedulerService::generate_schedules()` - イベント通知スケジュール生成は継続
- `SchedulerFacade::execute_recruitment_schedules()` - 定期募集は当面現状維持

## 移行のメリット

1. **自動実行**: 外部cronやFacade呼び出しが不要
2. **統一的な処理**: 通知も解散も同じSchedulerManagerで処理
3. **低遅延**: 10秒間隔のプリロードで即座に実行
4. **拡張性**: 新しいタスクタイプを追加しやすい
5. **整合性**: 実行時DB再確認パターンでデータ整合性を保証

## 移行のリスクと対策

**リスク1: 既存の通知が実行されない**
- 対策: 移行前にテストを実施
- 対策: SchedulerManagerとSchedulerFacadeを並行稼働させて検証

**リスク2: 重複実行**
- 対策: is_sentフラグで実行済みを管理
- 対策: 実行時DB再確認で重複を防止

**リスク3: last_process_timesとの整合性**
- 対策: Phase 1では通知のlast_process_times更新は維持
- 対策: 完全移行後にlast_process_timesテーブルから通知関連を削除

## 実装の優先順位

### 高優先度（今すぐ実施）
1. ✅ DissolutionTaskExecutor実装（完了）
2. ⏳ NotificationServiceとの統合
3. ⏳ SchedulerManagerの起動処理をBotに組み込み

### 中優先度（Phase 1完了後）
4. 既存のexecute_notifications()呼び出しを削除
5. last_process_timesの通知関連処理を削除

### 低優先度（将来）
6. 定期募集のscheduled_tasks化
7. DataCleanupTaskExecutorの実装

## 実装タスク

1. **NotificationRepositoryの拡張**
   - `find_by_datetime_range_with_txn()`メソッドの実装

2. **SchedulerManagerの拡張**
   - notificationsテーブルも処理するように実装
   - NotificationServiceとの統合

3. **既存コードの廃止予定マーク**
   - `SchedulerFacade::execute_notifications()` - `#[deprecated]`追加
   - `ScheduleNotificationTimer` - `#[deprecated]`追加
   - `SchedulerService::get_last_process_time()` - 注釈追加（定期募集では継続使用）
   - `SchedulerService::update_last_process_time()` - 注釈追加（定期募集では継続使用）

4. **Bot起動処理への統合**
   - main.rsまたはBot初期化処理でSchedulerManagerを起動
   - ScheduleNotificationTimerの起動を削除

5. **テストと検証**
   - SchedulerManagerの動作確認
   - 通知が正しく実行されることを確認
