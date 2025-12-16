use crate::repository::database::quest_repository::SeaOrmQuestRepository;
use crate::repository::quests_repository::QuestRepository;
use crate::services::quest::search::QuestSearchService;
use crate::types::PoiseContext;
use poise::serenity_prelude::AutocompleteChoice;
use sea_orm::DatabaseConnection;
use tracing::error;

/// クエスト名の入力候補を取得するファサード
///
/// オートコンプリートでクエスト名を検索する際に使用する。
/// Service層を使ってクエストを検索し、候補を返す。
pub async fn search_quests_for_autocomplete(
    ctx: PoiseContext<'_>,
    partial: &str,
) -> Vec<AutocompleteChoice> {
    // AppStateからDB接続を取得してRepositoryを作成
    let db_conn = ctx.data().app_state.guild_db().clone();
    let quest_repository = SeaOrmQuestRepository::new();

    // Service層を使って検索
    let search_service = QuestSearchService::new(&quest_repository);
    let results = search_service
        .search_for_autocomplete(&db_conn, partial)
        .await
        .unwrap_or_else(|e| {
            error!(error = %e, "クエスト検索に失敗しました");
            vec![]
        });

    // AutocompleteChoiceに変換（display_nameを表示、quest_nameを値として使用）
    results
        .into_iter()
        .map(|item| AutocompleteChoice::new(item.display_name, item.quest_name))
        .collect()
}

/// セレクトメニュー用にクエスト一覧（最大25件）を返す
pub async fn list_quests_for_select(ctx: PoiseContext<'_>) -> Vec<(String, i32)> {
    let db_conn = ctx.data().app_state.guild_db();
    let quest_repository = SeaOrmQuestRepository::new();

    match quest_repository.get_all(db_conn).await {
        Ok(list) => list.into_iter().take(25).map(|q| (q.name, q.id)).collect(),
        Err(e) => {
            error!(error = %e, "クエスト一覧の取得に失敗しました");
            vec![]
        }
    }
}

/// クエストIDから名称を取得
pub async fn get_quest_name_by_id(ctx: PoiseContext<'_>, quest_id: i32) -> Option<String> {
    let db_conn = ctx.data().app_state.guild_db();
    let quest_repository = SeaOrmQuestRepository::new();
    match quest_repository.get_by_target_id(db_conn, quest_id).await {
        Ok(Some(model)) => Some(model.name),
        _ => None,
    }
}

/// セレクトメニュー用にクエスト一覧（最大25件）を返す（DB直渡し版）
pub async fn list_quests_for_select_with_db(db: &DatabaseConnection) -> Vec<(String, i32)> {
    let quest_repository = SeaOrmQuestRepository::new();
    match quest_repository.get_all(db).await {
        Ok(list) => list.into_iter().take(25).map(|q| (q.name, q.id)).collect(),
        Err(e) => {
            error!(error = %e, "クエスト一覧の取得に失敗しました");
            vec![]
        }
    }
}

/// クエストIDから名称を取得（DB直渡し版）
pub async fn get_quest_name_by_id_with_db(
    db: &DatabaseConnection,
    quest_id: i32,
) -> Option<String> {
    let quest_repository = SeaOrmQuestRepository::new();
    match quest_repository.get_by_target_id(db, quest_id).await {
        Ok(Some(model)) => Some(model.name),
        _ => None,
    }
}
