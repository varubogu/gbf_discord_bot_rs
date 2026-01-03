use crate::repository::QuestRepository;
use crate::repository::quest_repository::QuestSearchResult;
use crate::types::Result;

/// オートコンプリート用のクエスト情報
#[derive(Debug, Clone)]
pub struct QuestAutocompleteItem {
    /// 表示名（エイリアスマッチの場合は "クエスト名 (エイリアス)" 形式）
    pub display_name: String,
    /// クエスト名（選択時に送信される値）
    pub quest_name: String,
}

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
    pub async fn search_for_autocomplete<C>(
        &self,
        db: &C,
        partial: &str,
    ) -> Result<Vec<QuestAutocompleteItem>>
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
            self.quest_repository
                .search_by_name_or_alias(db, partial)
                .await?
        };

        // Discord autocompleteは最大25件まで
        let limited_results: Vec<QuestAutocompleteItem> = results
            .into_iter()
            .take(25)
            .map(|r| create_autocomplete_item(&r))
            .collect();

        Ok(limited_results)
    }

    /// ギルド用のクエスト名またはエイリアスで部分一致検索を行い、結果を返す
    /// Discord autocompleteの制限に合わせて最大25件まで返す
    /// 空文字の場合はguild_questsで有効なクエストのみ、1文字以上の場合は全件対象
    pub async fn search_for_autocomplete_for_guild<C>(
        &self,
        db: &C,
        guild_id: i64,
        partial: &str,
    ) -> Result<Vec<QuestAutocompleteItem>>
    where
        C: sea_orm::ConnectionTrait,
    {
        let results = self
            .quest_repository
            .search_by_name_or_alias_for_guild(db, guild_id, partial)
            .await?;

        // Discord autocompleteは最大25件まで
        let limited_results: Vec<QuestAutocompleteItem> = results
            .into_iter()
            .take(25)
            .map(|r| create_autocomplete_item(&r))
            .collect();

        Ok(limited_results)
    }
}

/// 検索結果をオートコンプリートアイテムに変換
/// クエスト名とマッチしたテキストが異なる場合（エイリアスマッチの場合）は両方表示
fn create_autocomplete_item(result: &QuestSearchResult) -> QuestAutocompleteItem {
    let display_name = if result.name == result.matched_text {
        // クエスト名そのものがマッチした場合
        result.name.clone()
    } else {
        // エイリアスがマッチした場合はエイリアスも表示
        format!("{} ({})", result.name, result.matched_text)
    };

    QuestAutocompleteItem {
        display_name,
        quest_name: result.name.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::QuestRepository;
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};

    // テスト用のダミーDB接続
    // 実際のRepositoryはモックなので、このDB接続は使われない
    struct DummyDb;

    #[async_trait]
    impl sea_orm::ConnectionTrait for DummyDb {
        fn get_database_backend(&self) -> sea_orm::DatabaseBackend {
            sea_orm::DatabaseBackend::Postgres
        }

        async fn execute(
            &self,
            _stmt: sea_orm::Statement,
        ) -> std::result::Result<sea_orm::ExecResult, sea_orm::DbErr> {
            unimplemented!("テストでは使用されません")
        }

        async fn execute_unprepared(
            &self,
            _sql: &str,
        ) -> std::result::Result<sea_orm::ExecResult, sea_orm::DbErr> {
            unimplemented!("テストでは使用されません")
        }

        async fn query_one(
            &self,
            _stmt: sea_orm::Statement,
        ) -> std::result::Result<Option<sea_orm::QueryResult>, sea_orm::DbErr> {
            unimplemented!("テストでは使用されません")
        }

        async fn query_all(
            &self,
            _stmt: sea_orm::Statement,
        ) -> std::result::Result<Vec<sea_orm::QueryResult>, sea_orm::DbErr> {
            unimplemented!("テストでは使用されません")
        }
    }

    // テスト用の手動モックRepository
    // mockallはジェネリックライフタイムを正しく処理できないため、手動で実装
    struct MockQuestRepo {
        get_all_results: Arc<Mutex<Option<Result<Vec<crate::models::quests::Quest>>>>>,
        search_results: Arc<Mutex<Option<Result<Vec<QuestSearchResult>>>>>,
    }

    impl MockQuestRepo {
        fn new() -> Self {
            Self {
                get_all_results: Arc::new(Mutex::new(None)),
                search_results: Arc::new(Mutex::new(None)),
            }
        }

        fn expect_get_all(&self, result: Result<Vec<crate::models::quests::Quest>>) {
            *self.get_all_results.lock().unwrap() = Some(result);
        }

        fn expect_search_by_name_or_alias(&self, result: Result<Vec<QuestSearchResult>>) {
            *self.search_results.lock().unwrap() = Some(result);
        }
    }

    #[async_trait]
    impl QuestRepository for MockQuestRepo {
        async fn get_all<'c, C>(&self, _db: &'c C) -> Result<Vec<crate::models::quests::Quest>>
        where
            C: sea_orm::ConnectionTrait,
        {
            self.get_all_results
                .lock()
                .unwrap()
                .take()
                .expect("get_all was called but no expectation was set")
        }

        async fn get_by_target_id<'c, C>(
            &self,
            _db: &'c C,
            _target_id: i32,
        ) -> Result<Option<crate::models::quests::Quest>>
        where
            C: sea_orm::ConnectionTrait,
        {
            unimplemented!("このテストでは使用されません")
        }

        async fn search_by_name_or_alias<'c, C>(
            &self,
            _db: &'c C,
            _partial: &str,
        ) -> Result<Vec<QuestSearchResult>>
        where
            C: sea_orm::ConnectionTrait,
        {
            self.search_results
                .lock()
                .unwrap()
                .take()
                .expect("search_by_name_or_alias was called but no expectation was set")
        }

        async fn search_by_name_or_alias_for_guild<'c, C>(
            &self,
            _db: &'c C,
            _guild_id: i64,
            _partial: &str,
        ) -> Result<Vec<QuestSearchResult>>
        where
            C: sea_orm::ConnectionTrait,
        {
            unimplemented!("このテストでは使用されません")
        }

        async fn search_enabled_quests<'c, C>(
            &self,
            _db: &'c C,
            _guild_id: i64,
            _partial: &str,
        ) -> Result<Vec<QuestSearchResult>>
        where
            C: sea_orm::ConnectionTrait,
        {
            unimplemented!("このテストでは使用されません")
        }

        async fn search_disabled_quests<'c, C>(
            &self,
            _db: &'c C,
            _guild_id: i64,
            _partial: &str,
        ) -> Result<Vec<QuestSearchResult>>
        where
            C: sea_orm::ConnectionTrait,
        {
            unimplemented!("このテストでは使用されません")
        }
    }

    #[tokio::test]
    async fn test_search_for_autocomplete_empty_string() {
        let mock_repo = MockQuestRepo::new();
        // ダミーDB接続（実際には使われない）
        let dummy_db = DummyDb;

        // 空文字列の場合は全クエストを取得する
        mock_repo.expect_get_all(Ok(vec![
            crate::models::quests::Quest {
                id: 1,
                name: "クエスト1".to_string(),
                default_battle_style_id: 1,
                recruit_count: 30,
                available_battle_style_ids: "1,2".to_string(),
                sort_order: 0,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            crate::models::quests::Quest {
                id: 2,
                name: "クエスト2".to_string(),
                default_battle_style_id: 1,
                recruit_count: 30,
                available_battle_style_ids: "1,2".to_string(),
                sort_order: 0,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
        ]));

        let service = QuestSearchService::new(&mock_repo);

        let result = service.search_for_autocomplete(&dummy_db, "").await;
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].display_name, "クエスト1");
        assert_eq!(results[0].quest_name, "クエスト1");
        assert_eq!(results[1].display_name, "クエスト2");
        assert_eq!(results[1].quest_name, "クエスト2");
    }

    #[tokio::test]
    async fn test_search_for_autocomplete_with_results() {
        let mock_repo = MockQuestRepo::new();
        // ダミーDB接続（実際には使われない）
        let dummy_db = DummyDb;

        mock_repo.expect_search_by_name_or_alias(Ok(vec![
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
        ]));

        let service = QuestSearchService::new(&mock_repo);
        let result = service.search_for_autocomplete(&dummy_db, "test").await;

        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].display_name, "テストクエスト");
        assert_eq!(results[0].quest_name, "テストクエスト");
        assert_eq!(results[1].display_name, "サンプルクエスト (sample)");
        assert_eq!(results[1].quest_name, "サンプルクエスト");
    }

    #[tokio::test]
    async fn test_create_autocomplete_item_name_match() {
        let result = QuestSearchResult {
            quest_id: 1,
            name: "テストクエスト".to_string(),
            matched_text: "テストクエスト".to_string(),
        };

        let item = create_autocomplete_item(&result);
        assert_eq!(item.display_name, "テストクエスト");
        assert_eq!(item.quest_name, "テストクエスト");
    }

    #[tokio::test]
    async fn test_create_autocomplete_item_alias_match() {
        let result = QuestSearchResult {
            quest_id: 1,
            name: "テストクエスト".to_string(),
            matched_text: "test".to_string(),
        };

        let item = create_autocomplete_item(&result);
        assert_eq!(item.display_name, "テストクエスト (test)");
        assert_eq!(item.quest_name, "テストクエスト");
    }
}
