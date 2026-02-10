use crate::repository::QuestRepository;
use crate::repository::db_helper::set_current_guild_id;
use crate::services::quest::search::QuestSearchService;
use crate::services::recruitment::quest_query_service::QuestQueryService;
use crate::types::discord::AutocompleteOption;
use sea_orm::{DatabaseConnection, TransactionTrait};
use tracing::error;

/// クエスト名の入力候補を取得するファサード
///
/// オートコンプリートでクエスト名を検索する際に使用する。
/// Service層を使ってクエストを検索し、候補を返す。
/// guild_quest_disablesテーブルを考慮し、空文字の場合は無効化されたクエストを除外、1文字以上の場合は全件対象
///
/// # 引数
/// * `conn` - データベース接続
/// * `guild_id` - ギルドID
/// * `partial` - 部分一致検索文字列
pub async fn search_quests_for_autocomplete<R: QuestRepository>(
    conn: &DatabaseConnection,
    quest_repository: &R,
    guild_id: i64,
    partial: &str,
) -> Vec<AutocompleteOption> {
    // トランザクションを開始してguild_idを設定
    let txn = match conn.begin().await {
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
    let search_service = QuestSearchService::new(quest_repository);
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
        .map(|item| AutocompleteOption::new(item.display_name, item.quest_name))
        .collect()
}

/// セレクトメニュー用にクエスト一覧（最大25件）を返す
pub async fn list_quests_for_select<R: QuestRepository>(
    db: &DatabaseConnection,
    quest_repository: R,
) -> Vec<(String, i32)> {
    let service = QuestQueryService::new(quest_repository);
    match service.get_all_quests(db).await {
        Ok(list) => list.into_iter().take(25).map(|q| (q.name, q.id)).collect(),
        Err(e) => {
            error!(error = %e, "クエスト一覧の取得に失敗しました");
            vec![]
        }
    }
}

/// クエストIDから名称を取得
pub async fn get_quest_name_by_id<R: QuestRepository>(
    db: &DatabaseConnection,
    quest_repository: R,
    quest_id: i32,
) -> Option<String> {
    let service = QuestQueryService::new(quest_repository);
    match service.get_quest_by_id(db, quest_id).await {
        Ok(quest) => Some(quest.name),
        _ => None,
    }
}
