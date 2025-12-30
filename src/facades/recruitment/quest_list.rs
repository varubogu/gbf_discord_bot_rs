use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::repository::database::quest_repository::SeaOrmQuestRepository;
use crate::services::quest::search::QuestSearchService;
use crate::services::recruitment::quest_query_service::QuestQueryService;
use crate::types::PoiseContext;
use poise::serenity_prelude::AutocompleteChoice;
use sea_orm::{DatabaseConnection, TransactionTrait};
use tracing::error;

/// クエスト名の入力候補を取得するファサード
///
/// オートコンプリートでクエスト名を検索する際に使用する。
/// Service層を使ってクエストを検索し、候補を返す。
/// guild_quest_disablesテーブルを考慮し、空文字の場合は無効化されたクエストを除外、1文字以上の場合は全件対象
pub async fn search_quests_for_autocomplete(
    ctx: PoiseContext<'_>,
    partial: &str,
) -> Vec<AutocompleteChoice> {
    // ギルドIDを取得
    let guild_id = match ctx.guild_id() {
        Some(id) => id.get() as i64,
        None => {
            error!("ギルドIDが取得できませんでした");
            return vec![];
        }
    };

    // AppStateからDB接続を取得してRepositoryを作成
    let db_conn = ctx.data().app_state.guild_db().clone();
    let quest_repository = SeaOrmQuestRepository::new();

    // トランザクションを開始してguild_idを設定
    let txn = match db_conn.begin().await {
        Ok(t) => t,
        Err(e) => {
            error!(error = %e, "トランザクション開始に失敗しました");
            return vec![];
        }
    };

    // RLSポリシー用にセッション変数を設定
    if let Err(e) = set_current_guild_id(&txn, guild_id).await {
        error!(error = %e, "guild_idの設定に失敗しました");
        return vec![];
    }

    // Service層を使って検索
    let search_service = QuestSearchService::new(&quest_repository);
    let results = search_service
        .search_for_autocomplete_for_guild(&txn, guild_id, partial)
        .await
        .unwrap_or_else(|e| {
            error!(error = %e, "クエスト検索に失敗しました");
            vec![]
        });

    // 読み取り専用なのでロールバック（コミット不要）
    let _ = txn.rollback().await;

    // AutocompleteChoiceに変換（display_nameを表示、quest_nameを値として使用）
    results
        .into_iter()
        .map(|item| AutocompleteChoice::new(item.display_name, item.quest_name))
        .collect()
}

/// セレクトメニュー用にクエスト一覧（最大25件）を返す（DB直渡し版）
pub async fn list_quests_for_select_with_db(db: &DatabaseConnection) -> Vec<(String, i32)> {
    let service = QuestQueryService::new();
    match service.get_all_quests(db).await {
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
    let service = QuestQueryService::new();
    match service.get_quest_by_id(db, quest_id).await {
        Ok(quest) => Some(quest.name),
        _ => None,
    }
}
