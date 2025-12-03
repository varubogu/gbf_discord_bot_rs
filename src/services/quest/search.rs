use crate::repository::quests_repository::QuestSearchResult;
use crate::repository::QuestRepository;
use crate::types::Result;

/// クエスト検索サービス
/// クエスト名やエイリアスから部分一致検索を行う
pub struct QuestSearchService<'a, R: QuestRepository> {
    quest_repository: &'a R,
}

impl<'a, R: QuestRepository> QuestSearchService<'a, R> {
    pub fn new(quest_repository: &'a R) -> Self {
        Self { quest_repository }
    }

    /// クエスト名またはエイリアスで部分一致検索を行い、結果を返す
    /// Discord autocompleteの制限に合わせて最大25件まで返す
    pub async fn search_for_autocomplete<'c, C>(&self, db: &'c C, partial: &str) -> Result<Vec<String>>
    where
        C: sea_orm::ConnectionTrait,
    {
        let results = if partial.trim().is_empty() {
            // 空文字列の場合は全クエストを取得
            let all_quests = self.quest_repository.get_all(db).await?;
            all_quests
                .into_iter()
                .map(|q| QuestSearchResult {
                    quest_id: q.id,
                    name: q.name.clone(),
                    matched_text: q.name,
                })
                .collect()
        } else {
            // 部分一致検索
            self.quest_repository.search_by_name_or_alias(db, partial).await?
        };

        // Discord autocompleteは最大25件まで
        let limited_results: Vec<String> = results
            .into_iter()
            .take(25)
            .map(|r| format_search_result(&r))
            .collect();

        Ok(limited_results)
    }
}

/// 検索結果をDiscord autocompleteの表示用にフォーマット
/// クエスト名とマッチしたテキストが異なる場合（エイリアスマッチの場合）は両方表示
fn format_search_result(result: &QuestSearchResult) -> String {
    if result.name == result.matched_text {
        // クエスト名そのものがマッチした場合
        result.name.clone()
    } else {
        // エイリアスがマッチした場合はエイリアスも表示
        format!("{} ({})", result.name, result.matched_text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::QuestRepository;
    use async_trait::async_trait;
    use mockall::mock;

    mock! {
        QuestRepo {}

        #[async_trait]
        impl QuestRepository for QuestRepo {
            async fn get_all<'c, C>(&self, db: &'c C) -> Result<Vec<crate::models::quests::Quest>>
            where
                C: sea_orm::ConnectionTrait;
            async fn get_by_target_id<'c, C>(&self, db: &'c C, target_id: i32) -> Result<Option<crate::models::quests::Quest>>
            where
                C: sea_orm::ConnectionTrait;
            async fn search_by_name_or_alias<'c, C>(&self, db: &'c C, partial: &str) -> Result<Vec<QuestSearchResult>>
            where
                C: sea_orm::ConnectionTrait;
        }
    }

    #[tokio::test]
    async fn test_search_for_autocomplete_empty_string() {
        let mut mock_repo = MockQuestRepo::new();

        // 空文字列の場合は全クエストを取得する
        mock_repo.expect_get_all().returning(|| {
            Ok(vec![
                crate::models::quests::Quest {
                    id: 1,
                    name: "クエスト1".to_string(),
                    default_battle_style_id: 1,
                    recruit_count: 30,
                    available_battle_style_ids: "1,2".to_string(),
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                },
                crate::models::quests::Quest {
                    id: 2,
                    name: "クエスト2".to_string(),
                    default_battle_style_id: 1,
                    recruit_count: 30,
                    available_battle_style_ids: "1,2".to_string(),
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                },
            ])
        });

        let service = QuestSearchService::new(&mock_repo);

        let result = service.search_for_autocomplete("").await;
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0], "クエスト1");
        assert_eq!(results[1], "クエスト2");
    }

    #[tokio::test]
    async fn test_search_for_autocomplete_with_results() {
        let mut mock_repo = MockQuestRepo::new();

        mock_repo
            .expect_search_by_name_or_alias()
            .returning(|_| {
                Ok(vec![
                    QuestSearchResult {
                        quest_id: 1,
                        name: "テストクエスト".to_string(),
                        matched_text: "テストクエスト".to_string(),
                    },
                    QuestSearchResult {
                        quest_id: 2,
                        name: "サンプルクエスト".to_string(),
                        matched_text: "sample".to_string(),
                    },
                ])
            });

        let service = QuestSearchService::new(&mock_repo);
        let result = service.search_for_autocomplete("test").await;

        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0], "テストクエスト");
        assert_eq!(results[1], "サンプルクエスト (sample)");
    }

    #[tokio::test]
    async fn test_format_search_result_name_match() {
        let result = QuestSearchResult {
            quest_id: 1,
            name: "テストクエスト".to_string(),
            matched_text: "テストクエスト".to_string(),
        };

        let formatted = format_search_result(&result);
        assert_eq!(formatted, "テストクエスト");
    }

    #[tokio::test]
    async fn test_format_search_result_alias_match() {
        let result = QuestSearchResult {
            quest_id: 1,
            name: "テストクエスト".to_string(),
            matched_text: "test".to_string(),
        };

        let formatted = format_search_result(&result);
        assert_eq!(formatted, "テストクエスト (test)");
    }
}
