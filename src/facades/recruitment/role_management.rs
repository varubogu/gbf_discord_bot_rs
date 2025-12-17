use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::services::recruitment::quest_query_service::QuestQueryService;
use crate::services::recruitment::role_notification::RoleNotificationService;
use crate::types;
use crate::types::PoiseContext;
use sea_orm::TransactionTrait;
use tracing::{info, warn, instrument};

/// 募集通知ロールを追加するFacade
///
/// # 引数
/// * `ctx` - Poiseコンテキスト
/// * `quest_name_or_alias` - クエスト名またはエイリアス（"すべて"の場合は全募集通知）
/// * `role_ids` - 追加するロールIDのリスト（最大6個）
///
/// # 戻り値
/// 追加された個数
#[instrument(level = "debug", skip(ctx))]
pub async fn add_recruitment_notification_roles(
    ctx: &PoiseContext<'_>,
    quest_name_or_alias: &str,
    role_ids: Vec<u64>,
) -> types::Result<usize> {
    info!(
        quest_name_or_alias = quest_name_or_alias,
        role_count = role_ids.len(),
        "募集通知ロールを追加します"
    );

    let app_state = &ctx.data().app_state;
    let conn = app_state.guild_db();
    let txn = conn.begin().await?;

    // Discord固有情報を取得
    let guild_id = ctx.guild_id().map(|id| id.get()).unwrap_or(0);

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id as i64).await?;

    let result = async {
        let role_service = RoleNotificationService::new();
        let quest_query_service = QuestQueryService::new();

        let mut added_count = 0;

        // "すべて"の場合は全募集通知ロールとして登録
        if quest_name_or_alias.trim() == "すべて" {
            info!("全募集通知ロールとして登録します");
            for role_id in role_ids {
                let is_added = role_service
                    .add_all_recruitment_role(&txn, guild_id as i64, role_id as i64)
                    .await?;
                if is_added {
                    added_count += 1;
                }
            }
        } else {
            // クエスト名またはエイリアスでクエストIDを解決
            let quest = quest_query_service
                .search_and_get_quest_by_name(conn, quest_name_or_alias)
                .await?;

            let quest_id = quest.id;

            info!(quest_id = quest_id, "クエスト別募集通知ロールとして登録します");

            for role_id in role_ids {
                let is_added = role_service
                    .add_quest_recruitment_role(&txn, guild_id as i64, quest_id, role_id as i64)
                    .await?;
                if is_added {
                    added_count += 1;
                }
            }
        }

        Ok(added_count)
    }
    .await;

    match result {
        Ok(count) => {
            txn.commit().await?;
            info!(added_count = count, "募集通知ロールを追加しました");
            Ok(count)
        }
        Err(e) => {
            warn!(error = %e, "募集通知ロールの追加に失敗しました");
            txn.rollback().await?;
            Err(e)
        }
    }
}

/// 募集通知ロールを削除するFacade
///
/// # 引数
/// * `ctx` - Poiseコンテキスト
/// * `quest_name_or_alias` - クエスト名またはエイリアス（"すべて"の場合は全募集通知）
/// * `role_ids` - 削除するロールIDのリスト（最大6個）
///
/// # 戻り値
/// 削除された個数
#[instrument(level = "debug", skip(ctx))]
pub async fn remove_recruitment_notification_roles(
    ctx: &PoiseContext<'_>,
    quest_name_or_alias: &str,
    role_ids: Vec<u64>,
) -> types::Result<u64> {
    info!(
        quest_name_or_alias = quest_name_or_alias,
        role_count = role_ids.len(),
        "募集通知ロールを削除します"
    );

    let app_state = &ctx.data().app_state;
    let conn = app_state.guild_db();
    let txn = conn.begin().await?;

    // Discord固有情報を取得
    let guild_id = ctx.guild_id().map(|id| id.get()).unwrap_or(0);

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id as i64).await?;

    let result = async {
        let role_service = RoleNotificationService::new();
        let quest_query_service = QuestQueryService::new();

        let mut deleted_count: u64 = 0;

        // "すべて"の場合は全募集通知ロールから削除
        if quest_name_or_alias.trim() == "すべて" {
            info!("全募集通知ロールから削除します");
            for role_id in role_ids {
                let count = role_service
                    .remove_all_recruitment_role(&txn, guild_id as i64, role_id as i64)
                    .await?;
                deleted_count += count;
            }
        } else {
            // クエスト名またはエイリアスでクエストIDを解決
            let quest = quest_query_service
                .search_and_get_quest_by_name(conn, quest_name_or_alias)
                .await?;

            let quest_id = quest.id;

            info!(quest_id = quest_id, "クエスト別募集通知ロールから削除します");

            for role_id in role_ids {
                let count = role_service
                    .remove_quest_recruitment_role(&txn, guild_id as i64, quest_id, role_id as i64)
                    .await?;
                deleted_count += count;
            }
        }

        Ok(deleted_count)
    }
    .await;

    match result {
        Ok(count) => {
            txn.commit().await?;
            info!(deleted_count = count, "募集通知ロールを削除しました");
            Ok(count)
        }
        Err(e) => {
            warn!(error = %e, "募集通知ロールの削除に失敗しました");
            txn.rollback().await?;
            Err(e)
        }
    }
}
