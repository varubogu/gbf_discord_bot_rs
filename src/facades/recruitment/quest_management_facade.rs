//! クエスト管理Facade
//!
//! クエストの有効化/無効化および一覧取得を担当する。
//! トランザクション境界とRLSセッション設定は本Facadeで管理する。

use crate::infrastructure::database::session::set_current_guild_id;
use crate::repository::{GuildQuestDisableRepository, QuestRepository};
use crate::types::{AppState, Result};
use sea_orm::TransactionTrait;
use std::collections::HashSet;
use tracing::{error, info};

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

    let quest_repo = app_state.repositories.quest;
    let disable_repo = app_state.repositories.guild_quest_disable;

    let result = match filter {
        QuestListFilter::All => {
            let all_quests = quest_repo.get_all(&txn).await?;
            let disabled_ids = disable_repo.get_disabled_quest_ids(&txn, guild_id).await?;

            let statuses = all_quests
                .into_iter()
                .map(|quest| QuestStatusItem {
                    is_enabled: !disabled_ids.contains(&quest.id),
                    name: quest.name,
                })
                .collect();
            QuestListResult::All(statuses)
        }
        QuestListFilter::EnabledOnly => {
            let quests = quest_repo
                .search_enabled_quests(&txn, guild_id, "")
                .await?
                .into_iter()
                .map(|q| q.name)
                .collect();
            QuestListResult::Enabled(quests)
        }
        QuestListFilter::DisabledOnly => {
            let quests = quest_repo
                .search_disabled_quests(&txn, guild_id, "")
                .await?
                .into_iter()
                .map(|q| q.name)
                .collect();
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
    let unique_quest_names: Vec<String> = quest_names
        .into_iter()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    let conn = app_state.guild_db();
    let txn = conn.begin().await?;
    set_current_guild_id(&txn, guild_id).await?;

    let quest_repo = app_state.repositories.quest;
    let disable_repo = app_state.repositories.guild_quest_disable;

    let mut changed_count = 0;
    let mut already_in_target_state = Vec::new();
    let mut not_found = Vec::new();

    for quest_name in &unique_quest_names {
        let search_results = quest_repo.search_by_name_or_alias(&txn, quest_name).await?;
        let quest = match search_results.into_iter().find(|q| q.name == *quest_name) {
            Some(q) => q,
            None => {
                not_found.push(quest_name.clone());
                continue;
            }
        };

        let is_disabled = disable_repo
            .is_disabled(&txn, guild_id, quest.quest_id)
            .await?;
        let needs_update = match action {
            QuestStateChangeAction::Enable => is_disabled,
            QuestStateChangeAction::Disable => !is_disabled,
        };

        if !needs_update {
            already_in_target_state.push(quest_name.clone());
            continue;
        }

        match action {
            QuestStateChangeAction::Enable => {
                disable_repo
                    .enable_quest(&txn, guild_id, quest.quest_id)
                    .await?;
                info!(
                    guild_id,
                    quest_id = quest.quest_id,
                    quest_name = %quest_name,
                    "クエストを有効化しました"
                );
            }
            QuestStateChangeAction::Disable => {
                disable_repo
                    .disable_quest(&txn, guild_id, quest.quest_id)
                    .await?;
                info!(
                    guild_id,
                    quest_id = quest.quest_id,
                    quest_name = %quest_name,
                    "クエストを無効化しました"
                );
            }
        }

        changed_count += 1;
    }

    txn.commit().await?;
    Ok(QuestStateChangeResult {
        changed_count,
        already_in_target_state,
        not_found,
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

    let quest_repo = app_state.repositories.quest;
    let results = match filter {
        QuestListFilter::EnabledOnly => quest_repo.search_enabled_quests(&txn, guild_id, partial),
        QuestListFilter::DisabledOnly => quest_repo.search_disabled_quests(&txn, guild_id, partial),
        QuestListFilter::All => unreachable!("オートコンプリートでAllは使用しません"),
    }
    .await
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
