use chrono::{Duration, Utc};
use gbf_discord_bot_rs::infrastructure::database::repositories::recruitment::SeaOrmBattleRecruitmentsRepository;
use gbf_discord_bot_rs::infrastructure::database::repositories::schedule::{
    SeaOrmNotificationRepository, SeaOrmScheduledTaskRepository,
};
use gbf_discord_bot_rs::models::entities::worker::scheduled_tasks::{
    ScheduledTaskType, TaskExecutionStatus,
};
use gbf_discord_bot_rs::models::entities::worker::{
    battle_recruitments, notifications, scheduled_tasks,
};
use gbf_discord_bot_rs::services::maintenance::DataCleanupService;
use gbf_discord_bot_rs::types::DbRole;
use sea_orm::{ActiveModelTrait, EntityTrait, Set, TransactionTrait};

/// テスト用のAdminロール接続を取得
async fn get_test_admin_db() -> sea_orm::DatabaseConnection {
    gbf_discord_bot_rs::test_utils::connect_database_for_role(DbRole::Admin)
        .await
        .expect("テスト用Adminロールのデータベース接続に失敗しました")
}

#[tokio::test]
#[ignore] // 実際のDBが必要なため、デフォルトでは無効化
async fn test_data_cleanup_integration() {
    let db = get_test_admin_db().await;

    // リポジトリ初期化
    let recruitment_repo = SeaOrmBattleRecruitmentsRepository::new();
    let notification_repo = SeaOrmNotificationRepository::new();
    let task_repo = SeaOrmScheduledTaskRepository::new();

    let service = DataCleanupService::new(recruitment_repo, notification_repo, task_repo);

    // テストデータ作成（31日前の募集終了データ）
    let old_recruitment = battle_recruitments::ActiveModel {
        guild_id: Set(123456789),
        channel_id: Set(987654321),
        message_id: Set(111111111),
        quest_id: Set(1),
        battle_style_id: Set(1),
        quest_start_at: Set(Utc::now() - Duration::days(31)),
        is_recruiting: Set(false),
        is_canceled: Set(false),
        recruit_end_message_id: Set(None),
        full_notification_sent: Set(false),
        ..Default::default()
    };
    let inserted_recruitment = old_recruitment.insert(&db).await.unwrap();

    // テストデータ作成（31日前の送信済み通知）
    let old_notification = notifications::ActiveModel {
        guild_id: Set(123456789),
        channel_id: Set(987654321),
        message_text_id: Set("test_message".to_string()),
        is_sent: Set(true),
        ..Default::default()
    };
    let inserted_notification = old_notification.insert(&db).await.unwrap();

    // テストデータ作成（31日前の実行済みタスク）
    let old_task = scheduled_tasks::ActiveModel {
        schedule_datetime: Set(Utc::now() - Duration::days(31)),
        task_type: Set(ScheduledTaskType::Notification.as_i32()),
        guild_id: Set(Some(123456789)),
        channel_id: Set(Some(987654321)),
        execution_status: Set(TaskExecutionStatus::Succeeded),
        ..Default::default()
    };
    let inserted_task = old_task.insert(&db).await.unwrap();

    // テストデータ作成（1日前の募集終了データ - 削除されないはず）
    let recent_recruitment = battle_recruitments::ActiveModel {
        guild_id: Set(123456789),
        channel_id: Set(987654321),
        message_id: Set(222222222),
        quest_id: Set(1),
        battle_style_id: Set(1),
        quest_start_at: Set(Utc::now() - Duration::days(1)),
        is_recruiting: Set(false),
        is_canceled: Set(false),
        recruit_end_message_id: Set(None),
        full_notification_sent: Set(false),
        ..Default::default()
    };
    let recent_recruitment_id = recent_recruitment.insert(&db).await.unwrap().id;

    // トランザクション開始
    let txn = db.begin().await.unwrap();

    // クリーンアップ実行
    let stats = service.execute(&txn).await.unwrap();

    // トランザクションコミット
    txn.commit().await.unwrap();

    // 削除されたことを確認
    assert!(stats.deleted_recruitments >= 1);
    assert!(stats.deleted_notifications >= 1);
    assert!(stats.deleted_tasks >= 1);

    // 古いデータが削除されたことを確認
    let found_recruitment = battle_recruitments::Entity::find_by_id(inserted_recruitment.id)
        .one(&db)
        .await
        .unwrap();
    assert!(found_recruitment.is_none());

    let found_notification = notifications::Entity::find_by_id(inserted_notification.id)
        .one(&db)
        .await
        .unwrap();
    assert!(found_notification.is_none());

    let found_task = scheduled_tasks::Entity::find_by_id(inserted_task.id)
        .one(&db)
        .await
        .unwrap();
    assert!(found_task.is_none());

    // 最近のデータが削除されていないことを確認
    let found_recent = battle_recruitments::Entity::find_by_id(recent_recruitment_id)
        .one(&db)
        .await
        .unwrap();
    assert!(found_recent.is_some());

    // クリーンアップ
    battle_recruitments::Entity::delete_by_id(recent_recruitment_id)
        .exec(&db)
        .await
        .unwrap();
}

#[tokio::test]
#[ignore] // 実際のDBが必要なため、デフォルトでは無効化
async fn test_data_cleanup_does_not_delete_active_recruitment() {
    let db = get_test_admin_db().await;

    // リポジトリ初期化
    let recruitment_repo = SeaOrmBattleRecruitmentsRepository::new();
    let notification_repo = SeaOrmNotificationRepository::new();
    let task_repo = SeaOrmScheduledTaskRepository::new();

    let service = DataCleanupService::new(recruitment_repo, notification_repo, task_repo);

    // テストデータ作成（31日前だが募集中のデータ）
    let active_recruitment = battle_recruitments::ActiveModel {
        guild_id: Set(123456789),
        channel_id: Set(987654321),
        message_id: Set(333333333),
        quest_id: Set(1),
        battle_style_id: Set(1),
        quest_start_at: Set(Utc::now() - Duration::days(31)),
        is_recruiting: Set(true), // 募集中
        is_canceled: Set(false),
        recruit_end_message_id: Set(None),
        full_notification_sent: Set(false),
        ..Default::default()
    };
    let inserted = active_recruitment.insert(&db).await.unwrap();

    // トランザクション開始
    let txn = db.begin().await.unwrap();

    // クリーンアップ実行
    let _stats = service.execute(&txn).await.unwrap();

    // トランザクションコミット
    txn.commit().await.unwrap();

    // データが削除されていないことを確認
    let found = battle_recruitments::Entity::find_by_id(inserted.id)
        .one(&db)
        .await
        .unwrap();
    assert!(found.is_some());

    // クリーンアップ
    battle_recruitments::Entity::delete_by_id(inserted.id)
        .exec(&db)
        .await
        .unwrap();
}

#[tokio::test]
#[ignore] // 実際のDBが必要なため、デフォルトでは無効化
async fn test_data_cleanup_does_not_delete_unsent_notification() {
    let db = get_test_admin_db().await;

    // リポジトリ初期化
    let recruitment_repo = SeaOrmBattleRecruitmentsRepository::new();
    let notification_repo = SeaOrmNotificationRepository::new();
    let task_repo = SeaOrmScheduledTaskRepository::new();

    let service = DataCleanupService::new(recruitment_repo, notification_repo, task_repo);

    // テストデータ作成（31日前だが未送信の通知）
    let unsent_notification = notifications::ActiveModel {
        guild_id: Set(123456789),
        channel_id: Set(987654321),
        message_text_id: Set("test_message".to_string()),
        is_sent: Set(false), // 未送信
        ..Default::default()
    };
    let inserted = unsent_notification.insert(&db).await.unwrap();

    // トランザクション開始
    let txn = db.begin().await.unwrap();

    // クリーンアップ実行
    let _stats = service.execute(&txn).await.unwrap();

    // トランザクションコミット
    txn.commit().await.unwrap();

    // データが削除されていないことを確認
    let found = notifications::Entity::find_by_id(inserted.id)
        .one(&db)
        .await
        .unwrap();
    assert!(found.is_some());

    // クリーンアップ
    notifications::Entity::delete_by_id(inserted.id)
        .exec(&db)
        .await
        .unwrap();
}

#[tokio::test]
#[ignore] // 実際のDBが必要なため、デフォルトでは無効化
async fn test_data_cleanup_does_not_delete_data_cleanup_task() {
    let db = get_test_admin_db().await;

    // リポジトリ初期化
    let recruitment_repo = SeaOrmBattleRecruitmentsRepository::new();
    let notification_repo = SeaOrmNotificationRepository::new();
    let task_repo = SeaOrmScheduledTaskRepository::new();

    let service = DataCleanupService::new(recruitment_repo, notification_repo, task_repo);

    // テストデータ作成（31日前の実行済みDataCleanupタスク）
    let cleanup_task = scheduled_tasks::ActiveModel {
        schedule_datetime: Set(Utc::now() - Duration::days(31)),
        task_type: Set(ScheduledTaskType::DataCleanup.as_i32()),
        guild_id: Set(Some(123456789)),
        channel_id: Set(Some(987654321)),
        execution_status: Set(TaskExecutionStatus::Succeeded),
        ..Default::default()
    };
    let inserted = cleanup_task.insert(&db).await.unwrap();

    // トランザクション開始
    let txn = db.begin().await.unwrap();

    // クリーンアップ実行
    let _stats = service.execute(&txn).await.unwrap();

    // トランザクションコミット
    txn.commit().await.unwrap();

    // DataCleanupタスクが削除されていないことを確認
    let found = scheduled_tasks::Entity::find_by_id(inserted.id)
        .one(&db)
        .await
        .unwrap();
    assert!(found.is_some());

    // クリーンアップ
    scheduled_tasks::Entity::delete_by_id(inserted.id)
        .exec(&db)
        .await
        .unwrap();
}
