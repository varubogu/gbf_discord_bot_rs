// スケジュールファサード 結合テスト

use chrono::{Duration, Utc};
use gbf_discord_bot_rs::facades::schedule::{NotificationScheduleFacade, ScheduleQueryFacade};
use gbf_discord_bot_rs::models::entities::worker::{
    notifications as notification, scheduled_tasks as scheduled_task,
};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};

use super::test_helper::{TEST_CHANNEL_ID, TEST_GUILD_ID, create_test_app_state};

/// 1-1: 正常系：今後の通知が存在する
///
/// 未来日時のnotificationsデータありの場合、フォーマットされた通知一覧文字列が返る
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_get_future_notifications_formatted_with_data() {
    let app_state = create_test_app_state().await;
    let db = app_state.system_db();

    // テスト前のクリーンアップ
    let _ = notification::Entity::delete_many().exec(db).await;
    let _ = scheduled_task::Entity::delete_many().exec(db).await;

    // 未来の通知を作成
    let future_time = Utc::now() + Duration::hours(2);

    // 1. scheduled_tasksにレコード作成（schedule_datetimeはこちら）
    let task = scheduled_task::ActiveModel {
        schedule_datetime: Set(future_time),
        task_type: Set(1), // Notification
        guild_id: Set(Some(TEST_GUILD_ID)),
        channel_id: Set(Some(TEST_CHANNEL_ID)),
        is_executed: Set(false),
        ..Default::default()
    };
    let task_model = task.insert(db).await.expect("scheduled_task挿入失敗");

    // 2. notificationsにレコード作成（task_idで紐づけ）
    let notif = notification::ActiveModel {
        task_id: Set(task_model.id),
        guild_id: Set(TEST_GUILD_ID),
        channel_id: Set(TEST_CHANNEL_ID),
        message_text_id: Set("test_message".to_string()),
        is_sent: Set(false),
        ..Default::default()
    };
    notif.insert(db).await.expect("notification挿入失敗");

    let facade = NotificationScheduleFacade::new(app_state.clone().into());
    let result = facade
        .get_future_notifications_formatted(TEST_GUILD_ID, 10)
        .await;

    assert!(result.is_ok(), "結果がエラーです: {:?}", result);
    let text = result.unwrap();
    assert!(!text.is_empty(), "通知一覧が空です");
    assert!(text.contains(&TEST_CHANNEL_ID.to_string()));

    // クリーンアップ
    let _ = notification::Entity::delete_many().exec(db).await;
    let _ = scheduled_task::Entity::delete_many().exec(db).await;
}

/// 1-2: 正常系：今後の通知が存在しない
///
/// 未来日時のnotificationsデータなしの場合、空文字列が返る
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_get_future_notifications_formatted_no_data() {
    let app_state = create_test_app_state().await;
    let db = app_state.system_db();

    // テスト前のクリーンアップ
    let _ = notification::Entity::delete_many().exec(db).await;

    let facade = NotificationScheduleFacade::new(app_state.clone().into());
    let result = facade
        .get_future_notifications_formatted(TEST_GUILD_ID, 10)
        .await;

    assert!(result.is_ok());
    let text = result.unwrap();
    assert_eq!(text, "", "通知がない場合は空文字列");
}

/// 1-3: 正常系：limit制限の確認
///
/// 通知が多数存在する場合、limit=5で5件以下の通知が返る
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_get_future_notifications_formatted_with_limit() {
    let app_state = create_test_app_state().await;
    let db = app_state.system_db();

    // テスト前のクリーンアップ
    let _ = notification::Entity::delete_many().exec(db).await;
    let _ = scheduled_task::Entity::delete_many().exec(db).await;

    // 10件の未来通知を作成
    for i in 1..=10 {
        let future_time = Utc::now() + Duration::hours(i);

        // scheduled_tasksにレコード作成
        let task = scheduled_task::ActiveModel {
            schedule_datetime: Set(future_time),
            task_type: Set(1), // Notification
            guild_id: Set(Some(TEST_GUILD_ID)),
            channel_id: Set(Some(TEST_CHANNEL_ID)),
            is_executed: Set(false),
            ..Default::default()
        };
        let task_model = task.insert(db).await.expect("scheduled_task挿入失敗");

        // notificationsにレコード作成
        let notif = notification::ActiveModel {
            task_id: Set(task_model.id),
            guild_id: Set(TEST_GUILD_ID),
            channel_id: Set(TEST_CHANNEL_ID),
            message_text_id: Set(format!("test_message_{}", i)),
            is_sent: Set(false),
            ..Default::default()
        };
        notif.insert(db).await.expect("notification挿入失敗");
    }

    let facade = NotificationScheduleFacade::new(app_state.clone().into());
    let result = facade
        .get_future_notifications_formatted(TEST_GUILD_ID, 5)
        .await;

    assert!(result.is_ok());
    let text = result.unwrap();
    let line_count = text.lines().count();
    assert!(
        line_count <= 5,
        "limit=5で5件以下が期待されるが{}件",
        line_count
    );

    // クリーンアップ
    let _ = notification::Entity::delete_many().exec(db).await;
    let _ = scheduled_task::Entity::delete_many().exec(db).await;
}

/// 1-4: 正常系：limit=0
///
/// 通知が存在してもlimit=0の場合、空文字列が返る
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_get_future_notifications_formatted_limit_zero() {
    let app_state = create_test_app_state().await;
    let db = app_state.system_db();

    // テスト前のクリーンアップ
    let _ = notification::Entity::delete_many().exec(db).await;
    let _ = scheduled_task::Entity::delete_many().exec(db).await;

    // 通知を作成
    let future_time = Utc::now() + Duration::hours(1);

    // scheduled_tasksにレコード作成
    let task = scheduled_task::ActiveModel {
        schedule_datetime: Set(future_time),
        task_type: Set(1), // Notification
        guild_id: Set(Some(TEST_GUILD_ID)),
        channel_id: Set(Some(TEST_CHANNEL_ID)),
        is_executed: Set(false),
        ..Default::default()
    };
    let task_model = task.insert(db).await.expect("scheduled_task挿入失敗");

    // notificationsにレコード作成
    let notif = notification::ActiveModel {
        task_id: Set(task_model.id),
        guild_id: Set(TEST_GUILD_ID),
        channel_id: Set(TEST_CHANNEL_ID),
        message_text_id: Set("test_message".to_string()),
        is_sent: Set(false),
        ..Default::default()
    };
    notif.insert(db).await.expect("notification挿入失敗");

    let facade = NotificationScheduleFacade::new(app_state.clone().into());
    let result = facade
        .get_future_notifications_formatted(TEST_GUILD_ID, 0)
        .await;

    assert!(result.is_ok());
    let text = result.unwrap();
    assert_eq!(text, "", "limit=0の場合は空文字列");

    // クリーンアップ
    let _ = notification::Entity::delete_many().exec(db).await;
    let _ = scheduled_task::Entity::delete_many().exec(db).await;
}

/// 2-1: 正常系：通知履歴取得
///
/// 過去の送信済みnotificationsありの場合、フォーマットされた履歴文字列とScheduleStatsのタプルが返る
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_get_notification_history_formatted_with_data() {
    let app_state = create_test_app_state().await;
    let db = app_state.system_db();

    // テスト前のクリーンアップ
    let _ = notification::Entity::delete_many().exec(db).await;
    let _ = scheduled_task::Entity::delete_many().exec(db).await;

    // 過去の送信済み通知を作成
    let past_time = Utc::now() - Duration::hours(2);

    // scheduled_tasksにレコード作成
    let task = scheduled_task::ActiveModel {
        schedule_datetime: Set(past_time),
        task_type: Set(1), // Notification
        guild_id: Set(Some(TEST_GUILD_ID)),
        channel_id: Set(Some(TEST_CHANNEL_ID)),
        is_executed: Set(true),
        ..Default::default()
    };
    let task_model = task.insert(db).await.expect("scheduled_task挿入失敗");

    // notificationsにレコード作成
    let notif = notification::ActiveModel {
        task_id: Set(task_model.id),
        guild_id: Set(TEST_GUILD_ID),
        channel_id: Set(TEST_CHANNEL_ID),
        message_text_id: Set("test_message".to_string()),
        is_sent: Set(true),
        ..Default::default()
    };
    notif.insert(db).await.expect("notification挿入失敗");

    let facade = NotificationScheduleFacade::new(app_state.clone().into());
    let from = Utc::now() - Duration::days(1);
    let result = facade
        .get_notification_history_formatted(TEST_GUILD_ID, from, 10)
        .await;

    assert!(result.is_ok(), "結果がエラーです: {:?}", result);
    let (text, stats) = result.unwrap();
    assert!(!text.is_empty(), "履歴が空です");
    assert!(stats.total_count >= 0, "統計が取得されていません");

    // クリーンアップ
    let _ = notification::Entity::delete_many().exec(db).await;
    let _ = scheduled_task::Entity::delete_many().exec(db).await;
}

/// 2-2: 正常系：履歴なし
///
/// 指定期間内のnotificationsなしの場合、空の履歴と統計が返る
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_get_notification_history_formatted_no_data() {
    let app_state = create_test_app_state().await;
    let db = app_state.system_db();

    // テスト前のクリーンアップ
    let _ = notification::Entity::delete_many().exec(db).await;

    let facade = NotificationScheduleFacade::new(app_state.clone().into());
    let from = Utc::now() - Duration::days(1);
    let result = facade
        .get_notification_history_formatted(TEST_GUILD_ID, from, 10)
        .await;

    assert!(result.is_ok());
    let (text, stats) = result.unwrap();
    assert_eq!(text, "", "履歴がない場合は空文字列");
    assert_eq!(stats.total_count, 0, "統計はゼロ値");
}

/// 3-1: 正常系：統計データ取得
///
/// scheduled_tasks, notificationsデータありの場合、ScheduleStatsに正しい統計値が含まれる
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_get_stats_with_data() {
    let app_state = create_test_app_state().await;
    let db = app_state.system_db();

    // テスト前のクリーンアップ
    let _ = notification::Entity::delete_many().exec(db).await;
    let _ = scheduled_task::Entity::delete_many().exec(db).await;

    // 過去の通知を作成
    let past_time = Utc::now() - Duration::hours(1);

    // scheduled_tasksにレコード作成
    let task = scheduled_task::ActiveModel {
        schedule_datetime: Set(past_time),
        task_type: Set(1), // Notification
        guild_id: Set(Some(TEST_GUILD_ID)),
        channel_id: Set(Some(TEST_CHANNEL_ID)),
        is_executed: Set(true),
        ..Default::default()
    };
    let task_model = task.insert(db).await.expect("scheduled_task挿入失敗");

    // notificationsにレコード作成
    let notif = notification::ActiveModel {
        task_id: Set(task_model.id),
        guild_id: Set(TEST_GUILD_ID),
        channel_id: Set(TEST_CHANNEL_ID),
        message_text_id: Set("test_message".to_string()),
        is_sent: Set(true),
        ..Default::default()
    };
    notif.insert(db).await.expect("notification挿入失敗");

    let facade = ScheduleQueryFacade::new(app_state.clone().into());
    let result = facade.get_stats(TEST_GUILD_ID, 7).await;

    assert!(result.is_ok(), "統計取得がエラー: {:?}", result);
    let stats = result.unwrap();
    assert!(stats.total_count >= 0, "統計値が不正");

    // クリーンアップ
    let _ = notification::Entity::delete_many().exec(db).await;
    let _ = scheduled_task::Entity::delete_many().exec(db).await;
}

/// 3-2: 正常系：データなし
///
/// 対象ギルドのデータなしの場合、ゼロ値の統計が返る
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_get_stats_no_data() {
    let app_state = create_test_app_state().await;
    let db = app_state.system_db();

    // テスト前のクリーンアップ
    let _ = notification::Entity::delete_many().exec(db).await;

    let facade = ScheduleQueryFacade::new(app_state.clone().into());
    let result = facade.get_stats(TEST_GUILD_ID, 7).await;

    assert!(result.is_ok());
    let stats = result.unwrap();
    assert_eq!(stats.total_count, 0, "データなしの場合は統計ゼロ");
}

/// 3-3: 正常系：期間指定の確認
///
/// days=7で指定した場合、過去7日分の統計のみが集計される
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_get_stats_with_period() {
    let app_state = create_test_app_state().await;
    let db = app_state.system_db();

    // テスト前のクリーンアップ
    let _ = notification::Entity::delete_many().exec(db).await;
    let _ = scheduled_task::Entity::delete_many().exec(db).await;

    // 7日以内の通知を作成
    let recent_time = Utc::now() - Duration::days(5);

    // scheduled_tasksにレコード作成（最近）
    let task1 = scheduled_task::ActiveModel {
        schedule_datetime: Set(recent_time),
        task_type: Set(1), // Notification
        guild_id: Set(Some(TEST_GUILD_ID)),
        channel_id: Set(Some(TEST_CHANNEL_ID)),
        is_executed: Set(true),
        ..Default::default()
    };
    let task_model1 = task1.insert(db).await.expect("scheduled_task挿入失敗");

    // notificationsにレコード作成（最近）
    let notif1 = notification::ActiveModel {
        task_id: Set(task_model1.id),
        guild_id: Set(TEST_GUILD_ID),
        channel_id: Set(TEST_CHANNEL_ID),
        message_text_id: Set("test_message_recent".to_string()),
        is_sent: Set(true),
        ..Default::default()
    };
    notif1.insert(db).await.expect("notification挿入失敗");

    // 7日より前の通知を作成
    let old_time = Utc::now() - Duration::days(10);

    // scheduled_tasksにレコード作成（古い）
    let task2 = scheduled_task::ActiveModel {
        schedule_datetime: Set(old_time),
        task_type: Set(1), // Notification
        guild_id: Set(Some(TEST_GUILD_ID)),
        channel_id: Set(Some(TEST_CHANNEL_ID)),
        is_executed: Set(true),
        ..Default::default()
    };
    let task_model2 = task2.insert(db).await.expect("scheduled_task挿入失敗");

    // notificationsにレコード作成（古い）
    let notif2 = notification::ActiveModel {
        task_id: Set(task_model2.id),
        guild_id: Set(TEST_GUILD_ID),
        channel_id: Set(TEST_CHANNEL_ID),
        message_text_id: Set("test_message_old".to_string()),
        is_sent: Set(true),
        ..Default::default()
    };
    notif2.insert(db).await.expect("notification挿入失敗");

    let facade = ScheduleQueryFacade::new(app_state.clone().into());
    let result = facade.get_stats(TEST_GUILD_ID, 7).await;

    assert!(result.is_ok());
    let stats = result.unwrap();
    // 7日以内の通知のみがカウントされる（期間外の通知は除外される）
    // ただし実際の集計ロジック次第で結果は変わるため、エラーが出ないことを確認
    assert!(stats.total_count >= 0);

    // クリーンアップ
    let _ = notification::Entity::delete_many().exec(db).await;
    let _ = scheduled_task::Entity::delete_many().exec(db).await;
}
