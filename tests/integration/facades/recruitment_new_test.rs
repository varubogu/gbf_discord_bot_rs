// 新規募集ファサード 結合テスト
//
// 対象: src/facades/recruitment/new_recruit.rs

use gbf_discord_bot_rs::facades::recruitment::new_recruit;
use gbf_discord_bot_rs::models::entities::worker::battle_recruitments;
use gbf_discord_bot_rs::repository::db_helper::set_current_guild_id;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, TransactionTrait};
use std::sync::Arc;

use super::test_helper::{TEST_CHANNEL_ID, TEST_GUILD_ID, create_test_app_state};

/// テスト用ID（新規募集テスト専用）
const NEW_GUILD_ID: u64 = (TEST_GUILD_ID + 600) as u64;
const NEW_CHANNEL_ID: u64 = (TEST_CHANNEL_ID + 600) as u64;

/// 募集レコードを削除
async fn cleanup_recruitment(db: &sea_orm::DatabaseConnection, guild_id: i64, recruitment_id: i32) {
    // 関連する通知関連テーブルの削除
    use gbf_discord_bot_rs::models::entities::worker::notification_rel_battle_recruitments;

    let txn = db.begin().await.unwrap();
    set_current_guild_id(&txn, guild_id).await.unwrap();

    let _ = notification_rel_battle_recruitments::Entity::delete_many()
        .filter(notification_rel_battle_recruitments::Column::RecruitId.eq(recruitment_id))
        .exec(&txn)
        .await;

    let _ = battle_recruitments::Entity::delete_by_id(recruitment_id)
        .exec(&txn)
        .await;

    txn.commit().await.unwrap();
}

// =================================================
// update_message_id
// =================================================

/// 2-1: 正常系 - message_id更新
#[tokio::test]
#[ignore] // 実際のDBが必要
async fn test_update_message_id_success() {
    let app_state = Arc::new(create_test_app_state().await);

    // テスト用募集を直接作成
    use chrono::{Duration, Utc};
    use sea_orm::{ActiveModelTrait, Set};

    let model = battle_recruitments::ActiveModel {
        guild_id: Set(NEW_GUILD_ID as i64),
        channel_id: Set(NEW_CHANNEL_ID as i64),
        message_id: Set(0), // 仮のmessage_id
        quest_id: Set(1),
        battle_style_id: Set(1),
        quest_start_at: Set(Utc::now() + Duration::hours(24)),
        is_recruiting: Set(true),
        is_canceled: Set(false),
        recruit_end_message_id: Set(None),
        full_notification_sent: Set(false),
        ..Default::default()
    };
    let db = app_state.guild_db();
    let txn = db.begin().await.unwrap();
    set_current_guild_id(&txn, NEW_GUILD_ID as i64)
        .await
        .unwrap();
    let inserted = model.insert(&txn).await.unwrap();
    txn.commit().await.unwrap();

    // message_id更新
    let new_message_id = 12345678_u64;
    let result =
        new_recruit::update_message_id(&app_state, NEW_GUILD_ID, inserted.id, new_message_id).await;
    assert!(result.is_ok(), "message_id更新に失敗: {:?}", result.err());

    // DBで確認
    let txn = db.begin().await.unwrap();
    set_current_guild_id(&txn, NEW_GUILD_ID as i64)
        .await
        .unwrap();
    let updated = battle_recruitments::Entity::find_by_id(inserted.id)
        .one(&txn)
        .await
        .unwrap()
        .unwrap();
    txn.commit().await.unwrap();
    assert_eq!(updated.message_id, new_message_id as i64);

    // クリーンアップ
    cleanup_recruitment(app_state.guild_db(), NEW_GUILD_ID as i64, inserted.id).await;
}
