use crate::models::quests::Quest;
use crate::repository::database::quest_repository::SeaOrmQuestRepository;
use crate::repository::QuestRepository;
use crate::types::{AppError, Result};
use sea_orm::DatabaseConnection;
use tracing::debug;

/// クエスト情報クエリService
/// クエスト検索・取得の責務を持つ
pub struct QuestQueryService;

impl QuestQueryService {
    pub fn new() -> Self {
        Self
    }

    /// クエスト名またはエイリアスで検索し、最初の結果のクエスト詳細を取得
    pub async fn search_and_get_quest_by_name(
        &self,
        db: &DatabaseConnection,
        quest_name: &str,
    ) -> Result<Quest> {
        let quest_repository = SeaOrmQuestRepository::new();

        // クエスト名で検索
        let search_results = quest_repository
            .search_by_name_or_alias(db, quest_name)
            .await?;

        let quest_search_result = search_results.first().ok_or_else(|| {
            AppError::NotFound(format!("クエスト '{}' が見つかりませんでした", quest_name))
        })?;

        // クエスト詳細を取得
        let quest = quest_repository
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
    pub async fn get_quest_by_id(
        &self,
        db: &DatabaseConnection,
        quest_id: i32,
    ) -> Result<Quest> {
        let quest_repository = SeaOrmQuestRepository::new();

        let quest = quest_repository
            .get_by_target_id(db, quest_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!("クエストID {} が見つかりませんでした", quest_id))
            })?;

        debug!(quest_id = quest_id, "クエスト詳細を取得しました");

        Ok(quest)
    }
}
