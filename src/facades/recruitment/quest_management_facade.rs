//! クエスト管理Facade
//!
//! クエストの有効化/無効化および一覧取得を担当する。
//! トランザクション境界とRLSセッション設定は本Facadeで管理する。

use crate::infrastructure::database::session::set_current_guild_id;
use crate::services::recruitment::quest_management_service::QuestManagementService;
use crate::types::{AppState, Result};
use sea_orm::TransactionTrait;
use tracing::error;

/// クエスト一覧の絞り込み条件
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestListFilter {
    All,
    EnabledOnly,
    DisabledOnly,
}

/// オートコンプリート候補
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestAutocompleteItem {
    pub display_name: String,
    pub quest_name: String,
}

/// 一覧表示用のクエスト状態
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestStatusItem {
    pub name: String,
    pub is_enabled: bool,
}

/// クエスト一覧結果
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuestListResult {
    All(Vec<QuestStatusItem>),
    Enabled(Vec<String>),
    Disabled(Vec<String>),
}

/// クエスト状態変更アクション
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestStateChangeAction {
    Enable,
    Disable,
}

/// クエスト状態変更結果
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestStateChangeResult {
    pub changed_count: usize,
    pub already_in_target_state: Vec<String>,
    pub not_found: Vec<String>,
}

/// 有効クエストのオートコンプリート候補を取得
pub async fn search_enabled_quests_for_autocomplete(
    app_state: &AppState,
    guild_id: i64,
    partial: &str,
) -> Vec<QuestAutocompleteItem> {
    search_for_autocomplete(app_state, guild_id, partial, QuestListFilter::EnabledOnly).await
}

/// 無効クエストのオートコンプリート候補を取得
pub async fn search_disabled_quests_for_autocomplete(
    app_state: &AppState,
    guild_id: i64,
    partial: &str,
) -> Vec<QuestAutocompleteItem> {
    search_for_autocomplete(app_state, guild_id, partial, QuestListFilter::DisabledOnly).await
}

/// クエスト一覧を取得
pub async fn list_quests(
    app_state: &AppState,
    guild_id: i64,
    filter: QuestListFilter,
) -> Result<QuestListResult> {
    let conn = app_state.guild_db();
    let txn = conn.begin().await?;
    set_current_guild_id(&txn, guild_id).await?;

    let service = QuestManagementService::new(
        app_state.repositories.quest,
        app_state.repositories.guild_quest_disable,
    );

    let result = match filter {
        QuestListFilter::All => {
            let statuses = service
                .list_all_with_enabled_flag(&txn, guild_id)
                .await?
                .into_iter()
                .map(|(name, is_enabled)| QuestStatusItem { name, is_enabled })
                .collect();
            QuestListResult::All(statuses)
        }
        QuestListFilter::EnabledOnly => {
            let quests = service.list_enabled_names(&txn, guild_id).await?;
            QuestListResult::Enabled(quests)
        }
        QuestListFilter::DisabledOnly => {
            let quests = service.list_disabled_names(&txn, guild_id).await?;
            QuestListResult::Disabled(quests)
        }
    };

    txn.rollback().await?;
    Ok(result)
}

/// クエストの有効/無効状態を変更
pub async fn change_quest_state(
    app_state: &AppState,
    guild_id: i64,
    quest_names: Vec<String>,
    action: QuestStateChangeAction,
) -> Result<QuestStateChangeResult> {
    let conn = app_state.guild_db();
    let txn = conn.begin().await?;
    set_current_guild_id(&txn, guild_id).await?;

    let service = QuestManagementService::new(
        app_state.repositories.quest,
        app_state.repositories.guild_quest_disable,
    );
    let summary = service
        .change_quest_state(
            &txn,
            guild_id,
            quest_names,
            matches!(action, QuestStateChangeAction::Enable),
        )
        .await?;
    txn.commit().await?;

    Ok(QuestStateChangeResult {
        changed_count: summary.changed_count,
        already_in_target_state: summary.already_in_target_state,
        not_found: summary.not_found,
    })
}

async fn search_for_autocomplete(
    app_state: &AppState,
    guild_id: i64,
    partial: &str,
    filter: QuestListFilter,
) -> Vec<QuestAutocompleteItem> {
    let conn = app_state.guild_db();
    let txn = match conn.begin().await {
        Ok(txn) => txn,
        Err(e) => {
            error!(error = %e, "トランザクション開始に失敗しました");
            return vec![];
        }
    };

    if let Err(e) = set_current_guild_id(&txn, guild_id).await {
        error!(error = %e, "guild_idの設定に失敗しました");
        return vec![];
    }

    let service = QuestManagementService::new(
        app_state.repositories.quest,
        app_state.repositories.guild_quest_disable,
    );
    let results = match filter {
        QuestListFilter::EnabledOnly => service.search_enabled(&txn, guild_id, partial).await,
        QuestListFilter::DisabledOnly => service.search_disabled(&txn, guild_id, partial).await,
        QuestListFilter::All => unreachable!("オートコンプリートでAllは使用しません"),
    }
    .unwrap_or_else(|e| {
        error!(error = %e, "クエスト検索に失敗しました");
        vec![]
    });

    let _ = txn.rollback().await;

    results
        .into_iter()
        .take(25)
        .map(|r| QuestAutocompleteItem {
            display_name: if r.name == r.matched_text {
                r.name.clone()
            } else {
                format!("{} ({})", r.name, r.matched_text)
            },
            quest_name: r.name,
        })
        .collect()
}
