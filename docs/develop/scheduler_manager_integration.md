# SchedulerManager統合ガイド

## 概要

SchedulerManagerは、scheduled_tasksとnotificationsの両方を処理する統一スケジューラーです。
既存のScheduleNotificationTimerを置き換えます。

## Bot起動時の統合方法

### 1. 必要な依存関係の準備

```rust
use gbf_discord_bot_rs::repository::database::{
    battle_recruitments_repository::BattleRecruitmentsRepositoryImpl,
    recruitment_participants_repository::RecruitmentParticipantsRepositoryImpl,
    schedule::{
        NotificationRepository,
        ScheduledTaskDissolutionRepository,
        ScheduledTaskRepository,
    },
};
use gbf_discord_bot_rs::services::{
    message::MessageService,
    schedule::SchedulerManager,
};
use std::sync::Arc;
```

### 2. SchedulerManagerの初期化と起動

```rust
// main.rsのBot起動処理内

// Repositoryの初期化
let task_repo = Arc::new(ScheduledTaskRepository::new());
let dissolution_repo = Arc::new(ScheduledTaskDissolutionRepository::new());
let notification_repo = Arc::new(NotificationRepository::new());
let recruitment_repo = Arc::new(BattleRecruitmentsRepositoryImpl::new());
let participants_repo = Arc::new(RecruitmentParticipantsRepositoryImpl::new());
let message_service = Arc::new(MessageService::new());

// SchedulerManagerの作成
let mut scheduler_manager = SchedulerManager::new(
    Arc::new(app_state.system_db().clone()),  // DatabaseConnection
    client.http.clone(),                       // Arc<Http>
    task_repo,
    dissolution_repo,
    notification_repo,
    recruitment_repo,
    participants_repo,
    message_service,
).await?;

// SchedulerManagerをバックグラウンドで起動
tokio::spawn(async move {
    if let Err(e) = scheduler_manager.start().await {
        error!(error = %e, "SchedulerManagerの起動に失敗しました");
    }

    // Bot終了時のクリーンアップ（シグナルハンドリングが必要）
    // scheduler_manager.stop().await
});
```

### 3. 既存のScheduleNotificationTimerの削除

```rust
// main.rsから以下のコードを削除またはコメントアウト

// スケジュール通知タイマーをバックグラウンドで起動
let app_state_for_scheduler = std::sync::Arc::new(app_state.clone());
let http = client.http.clone();
tokio::spawn(async move {
    let timer = std::sync::Arc::new(ScheduleNotificationTimer::new(
        app_state_for_scheduler,
        http,
    ));
    timer.start().await;
});
```

## 動作の違い

### 旧システム（ScheduleNotificationTimer）

- **実行間隔**: 10秒ごと
- **処理対象**: notificationsテーブルのみ
- **処理方法**: last_process_timesを使用して差分処理
- **問題点**: scheduled_tasksは別途処理が必要

### 新システム（SchedulerManager）

- **実行間隔**: 10秒ごと（同じ）
- **処理対象**: notificationsテーブル + scheduled_tasksテーブル
- **処理方法**: プリロード方式（20秒先まで）+ 実行時DB再確認
- **利点**: 統一的な処理、低遅延、拡張性

## トラブルシューティング

### Q: SchedulerManagerが起動しない

A: 以下を確認してください:
- DatabaseConnectionが正しく設定されているか
- すべてのRepositoryが正しく初期化されているか
- tokio-cron-schedulerの依存関係が追加されているか（Cargo.toml）

### Q: 通知が実行されない

A: ログを確認してください:
- "タスクと通知をプリロードしました" が表示されているか
- "通知を実行します" が表示されているか
- エラーログが出力されていないか

### Q: 既存の通知処理との重複実行

A: 移行期間中は以下のいずれかを選択してください:
1. ScheduleNotificationTimerをコメントアウト（推奨）
2. 両方を並行稼働して検証後、旧システムを削除

## 参照

- [設計書](./design/features/schedule_processing_system.md)
- [実装サマリー](./implementation_summary.md)
- [移行計画](./migration_plan.md)
