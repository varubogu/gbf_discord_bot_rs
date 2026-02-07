use crate::models::quests::Quest;
use crate::repository::QuestRepository;
use crate::types::{AppError, Result};
use sea_orm::DatabaseConnection;
use tracing::debug;

/// クエスト情報クエリService
/// クエスト検索・取得の責務を持つ
pub struct QuestQueryService<Q>
where
    Q: QuestRepository,
{
    quest_repository: Q,
}

impl<Q> QuestQueryService<Q>
where
    Q: QuestRepository,
{
    pub fn new(quest_repository: Q) -> Self {
        Self { quest_repository }
    }

    /// クエスト名またはエイリアスで検索し、最初の結果のクエスト詳細を取得
    pub async fn search_and_get_quest_by_name(
        &self,
        db: &DatabaseConnection,
        quest_name: &str,
    ) -> Result<Quest> {
        // クエスト名で検索
        let search_results = self
            .quest_repository
            .search_by_name_or_alias(db, quest_name)
            .await?;

        let quest_search_result = search_results.first().ok_or_else(|| {
            AppError::NotFound(format!("クエスト '{quest_name}' が見つかりませんでした"))
        })?;

        // クエスト詳細を取得
        let quest = self
            .quest_repository
            .get_by_target_id(db, quest_search_result.quest_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "クエストID {} の詳細情報が見つかりませんでした",
                    quest_search_result.quest_id
                ))
            })?;

        debug!(
            quest_name = quest_name,
            quest_id = quest.id,
            "クエストを検索・取得しました"
        );

        Ok(quest)
    }

    /// クエストIDでクエスト詳細を取得
    pub async fn get_quest_by_id(&self, db: &DatabaseConnection, quest_id: i32) -> Result<Quest> {
        let quest = self
            .quest_repository
            .get_by_target_id(db, quest_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!("クエストID {quest_id} が見つかりませんでした"))
            })?;

        debug!(quest_id = quest_id, "クエスト詳細を取得しました");

        Ok(quest)
    }

    /// すべてのクエストを取得
    pub async fn get_all_quests(&self, db: &DatabaseConnection) -> Result<Vec<Quest>> {
        let quests = self.quest_repository.get_all(db).await?;

        debug!(count = quests.len(), "クエスト一覧を取得しました");

        Ok(quests)
    }
}
