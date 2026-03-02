use crate::repository::{GuildQuestDisableRepository, QuestRepository};
use crate::types::Result;
use sea_orm::DatabaseTransaction;
use std::collections::HashSet;
use tracing::info;

/// クエスト状態変更結果
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestStateChangeSummary {
    pub changed_count: usize,
    pub already_in_target_state: Vec<String>,
    pub not_found: Vec<String>,
}

/// クエスト管理サービス
///
/// クエスト一覧取得・検索・有効/無効切り替えの業務ロジックを担当する。
pub struct QuestManagementService<Q, D>
where
    Q: QuestRepository,
    D: GuildQuestDisableRepository,
{
    quest_repo: Q,
    disable_repo: D,
}

impl<Q, D> QuestManagementService<Q, D>
where
    Q: QuestRepository,
    D: GuildQuestDisableRepository,
{
    pub fn new(quest_repo: Q, disable_repo: D) -> Self {
        Self {
            quest_repo,
            disable_repo,
        }
    }

    /// 有効クエスト検索
    pub async fn search_enabled(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        partial: &str,
    ) -> Result<Vec<crate::repository::quest_repository::QuestSearchResult>> {
        self.quest_repo
            .search_enabled_quests(txn, guild_id, partial)
            .await
    }

    /// 無効クエスト検索
    pub async fn search_disabled(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        partial: &str,
    ) -> Result<Vec<crate::repository::quest_repository::QuestSearchResult>> {
        self.quest_repo
            .search_disabled_quests(txn, guild_id, partial)
            .await
    }

    /// 全クエストと無効化情報を取得
    pub async fn list_all_with_enabled_flag(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
    ) -> Result<Vec<(String, bool)>> {
        let all_quests = self.quest_repo.get_all(txn).await?;
        let disabled_ids = self
            .disable_repo
            .get_disabled_quest_ids(txn, guild_id)
            .await?;

        Ok(all_quests
            .into_iter()
            .map(|quest| (quest.name, !disabled_ids.contains(&quest.id)))
            .collect())
    }

    /// 有効クエスト名一覧を取得
    pub async fn list_enabled_names(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
    ) -> Result<Vec<String>> {
        Ok(self
            .quest_repo
            .search_enabled_quests(txn, guild_id, "")
            .await?
            .into_iter()
            .map(|quest| quest.name)
            .collect())
    }

    /// 無効クエスト名一覧を取得
    pub async fn list_disabled_names(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
    ) -> Result<Vec<String>> {
        Ok(self
            .quest_repo
            .search_disabled_quests(txn, guild_id, "")
            .await?
            .into_iter()
            .map(|quest| quest.name)
            .collect())
    }

    /// クエスト状態を変更
    ///
    /// `enable=true` は有効化、`false` は無効化。
    pub async fn change_quest_state(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        quest_names: Vec<String>,
        enable: bool,
    ) -> Result<QuestStateChangeSummary> {
        let unique_quest_names: Vec<String> = quest_names
            .into_iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        let mut changed_count = 0;
        let mut already_in_target_state = Vec::new();
        let mut not_found = Vec::new();

        for quest_name in &unique_quest_names {
            let search_results = self
                .quest_repo
                .search_by_name_or_alias(txn, quest_name)
                .await?;
            let quest = match search_results.into_iter().find(|q| q.name == *quest_name) {
                Some(q) => q,
                None => {
                    not_found.push(quest_name.clone());
                    continue;
                }
            };

            let is_disabled = self
                .disable_repo
                .is_disabled(txn, guild_id, quest.quest_id)
                .await?;
            let needs_update = if enable { is_disabled } else { !is_disabled };

            if !needs_update {
                already_in_target_state.push(quest_name.clone());
                continue;
            }

            if enable {
                self.disable_repo
                    .enable_quest(txn, guild_id, quest.quest_id)
                    .await?;
                info!(
                    guild_id = guild_id,
                    quest_id = quest.quest_id,
                    quest_name = %quest_name,
                    "クエストを有効化しました"
                );
            } else {
                self.disable_repo
                    .disable_quest(txn, guild_id, quest.quest_id)
                    .await?;
                info!(
                    guild_id = guild_id,
                    quest_id = quest.quest_id,
                    quest_name = %quest_name,
                    "クエストを無効化しました"
                );
            }

            changed_count += 1;
        }

        Ok(QuestStateChangeSummary {
            changed_count,
            already_in_target_state,
            not_found,
        })
    }
}
