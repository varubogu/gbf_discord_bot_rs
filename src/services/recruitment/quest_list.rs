use crate::repository::QuestRepository;
use crate::services::quest::search::QuestSearchService;
use crate::services::recruitment::quest_query_service::QuestQueryService;
use crate::types::{AppError, Result};

/// 募集機能向けクエスト一覧サービス
///
/// オートコンプリート・選択肢生成・ID解決を担当する。
pub struct QuestListService<Q>
where
    Q: QuestRepository + Clone,
{
    quest_repository: Q,
}

impl<Q> QuestListService<Q>
where
    Q: QuestRepository + Clone,
{
    pub fn new(quest_repository: Q) -> Self {
        Self { quest_repository }
    }

    /// ギルド向けオートコンプリート候補を取得
    pub async fn search_for_autocomplete_for_guild<C>(
        &self,
        db: &C,
        guild_id: i64,
        partial: &str,
    ) -> Result<Vec<crate::services::quest::search::QuestAutocompleteItem>>
    where
        C: sea_orm::ConnectionTrait,
    {
        let search_service = QuestSearchService::new(&self.quest_repository);
        search_service
            .search_for_autocomplete_for_guild(db, guild_id, partial)
            .await
    }

    /// セレクトメニュー向けクエスト一覧を取得
    pub async fn list_for_select<C>(&self, db: &C, limit: usize) -> Result<Vec<(String, i32)>>
    where
        C: sea_orm::ConnectionTrait,
    {
        let query_service = QuestQueryService::new(self.quest_repository.clone());
        let quests = query_service.get_all_quests(db).await?;
        Ok(quests
            .into_iter()
            .take(limit)
            .map(|quest| (quest.name, quest.id))
            .collect())
    }

    /// クエストIDから名称を取得
    pub async fn get_name_by_id<C>(&self, db: &C, quest_id: i32) -> Result<Option<String>>
    where
        C: sea_orm::ConnectionTrait,
    {
        let query_service = QuestQueryService::new(self.quest_repository.clone());
        match query_service.get_quest_by_id(db, quest_id).await {
            Ok(quest) => Ok(Some(quest.name)),
            Err(AppError::NotFound(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
