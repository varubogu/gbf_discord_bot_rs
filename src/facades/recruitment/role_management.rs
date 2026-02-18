use crate::infrastructure::database::session::set_current_guild_id;
use crate::repository::QuestRepository;
use crate::services::recruitment::quest_query_service::QuestQueryService;
use crate::services::recruitment::role_notification::RoleNotificationService;
use crate::types;
use crate::types::AppState;
use sea_orm::TransactionTrait;
use std::collections::HashMap;
use tracing::{info, instrument, warn};

/// 募集通知ロール設定情報
#[derive(Debug, Clone)]
pub struct RecruitmentRoleSettings {
    /// 全募集通知ロール
    pub all_recruitment_roles: Vec<i64>,
    /// クエスト別募集通知ロール（クエストID → ロールIDリスト）
    pub quest_recruitment_roles: HashMap<i32, Vec<i64>>,
    /// クエスト情報（クエストID → クエスト名）
    pub quest_names: HashMap<i32, String>,
}

/// 募集通知ロールを追加するFacade
///
/// # 引数
/// * `app_state` - アプリケーション状態
/// * `guild_id` - ギルドID
/// * `quest_name_or_alias` - クエスト名またはエイリアス（"すべて"の場合は全募集通知）
/// * `role_ids` - 追加するロールIDのリスト（最大6個）
///
/// # 戻り値
/// 追加された個数
#[instrument(level = "debug", skip(app_state))]
pub async fn add_recruitment_notification_roles(
    app_state: &AppState,
    guild_id: u64,
    quest_name_or_alias: &str,
    role_ids: Vec<u64>,
) -> types::Result<usize> {
    info!(
        quest_name_or_alias = quest_name_or_alias,
        role_count = role_ids.len(),
        "募集通知ロールを追加します"
    );

    let conn = app_state.guild_db();
    let txn = conn.begin().await?;

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id as i64).await?;

    let result = async {
        let all_roles_repo = app_state.repositories.all_recruitment_notification_roles;
        let quest_roles_repo = app_state.repositories.quest_recruitment_notification_roles;
        let role_service = RoleNotificationService::new(all_roles_repo, quest_roles_repo);
        let quest_query_service = QuestQueryService::new(app_state.repositories.quest);

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

            info!(
                quest_id = quest_id,
                "クエスト別募集通知ロールとして登録します"
            );

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
/// * `app_state` - アプリケーション状態
/// * `guild_id` - ギルドID
/// * `quest_name_or_alias` - クエスト名またはエイリアス（"すべて"の場合は全募集通知）
/// * `role_ids` - 削除するロールIDのリスト（最大6個）
///
/// # 戻り値
/// 削除された個数
#[instrument(level = "debug", skip(app_state))]
pub async fn remove_recruitment_notification_roles(
    app_state: &AppState,
    guild_id: u64,
    quest_name_or_alias: &str,
    role_ids: Vec<u64>,
) -> types::Result<u64> {
    info!(
        quest_name_or_alias = quest_name_or_alias,
        role_count = role_ids.len(),
        "募集通知ロールを削除します"
    );

    let conn = app_state.guild_db();
    let txn = conn.begin().await?;

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id as i64).await?;

    let result = async {
        let all_roles_repo = app_state.repositories.all_recruitment_notification_roles;
        let quest_roles_repo = app_state.repositories.quest_recruitment_notification_roles;
        let role_service = RoleNotificationService::new(all_roles_repo, quest_roles_repo);
        let quest_query_service = QuestQueryService::new(app_state.repositories.quest);

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

            info!(
                quest_id = quest_id,
                "クエスト別募集通知ロールから削除します"
            );

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

/// 募集通知ロール設定を取得するFacade
///
/// # 引数
/// * `app_state` - アプリケーション状態
/// * `guild_id` - ギルドID
///
/// # 戻り値
/// 募集通知ロール設定情報
#[instrument(level = "debug", skip(app_state))]
pub async fn show_recruitment_notification_roles(
    app_state: &AppState,
    guild_id: u64,
) -> types::Result<RecruitmentRoleSettings> {
    info!("募集通知ロール設定を取得します");

    let conn = app_state.guild_db();
    let txn = conn.begin().await?;

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id as i64).await?;

    let result = async {
        let all_roles_repo = app_state.repositories.all_recruitment_notification_roles;
        let quest_roles_repo = app_state.repositories.quest_recruitment_notification_roles;
        let role_service = RoleNotificationService::new(all_roles_repo, quest_roles_repo);
        let quest_repo = app_state.repositories.quest;

        // 全募集通知ロール取得
        let all_roles = role_service
            .get_all_recruitment_roles(&txn, guild_id as i64)
            .await?;

        // クエスト別募集通知ロール取得
        let quest_roles = role_service
            .get_quest_recruitment_roles(&txn, guild_id as i64)
            .await?;

        // クエスト情報取得
        let all_quests = quest_repo.get_all(conn).await?;
        let quest_names: HashMap<i32, String> = all_quests
            .into_iter()
            .map(|q| (q.id, q.name.clone()))
            .collect();

        // クエストID別にロールをグループ化
        let mut quest_role_map: HashMap<i32, Vec<i64>> = HashMap::new();
        for role in quest_roles {
            quest_role_map
                .entry(role.quest_id)
                .or_default()
                .push(role.role_id);
        }

        Ok(RecruitmentRoleSettings {
            all_recruitment_roles: all_roles.into_iter().map(|r| r.role_id).collect(),
            quest_recruitment_roles: quest_role_map,
            quest_names,
        })
    }
    .await;

    match result {
        Ok(settings) => {
            txn.commit().await?;
            info!("募集通知ロール設定を取得しました");
            Ok(settings)
        }
        Err(e) => {
            warn!(error = %e, "募集通知ロール設定の取得に失敗しました");
            txn.rollback().await?;
            Err(e)
        }
    }
}
