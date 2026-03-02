use crate::infrastructure::database::session::set_current_guild_id;
use crate::services::recruitment::quest_list::QuestListService;
use crate::types::AppState;
use crate::types::discord::AutocompleteOption;
use sea_orm::TransactionTrait;
use tracing::error;

/// クエスト名の入力候補を取得するファサード
///
/// オートコンプリートでクエスト名を検索する際に使用する。
/// Service層を使ってクエストを検索し、候補を返す。
/// guild_quest_disablesテーブルを考慮し、空文字の場合は無効化されたクエストを除外、1文字以上の場合は全件対象
///
/// # 引数
/// * `app_state` - アプリケーション状態
/// * `guild_id` - ギルドID
/// * `partial` - 部分一致検索文字列
pub async fn search_quests_for_autocomplete(
    app_state: &AppState,
    guild_id: i64,
    partial: &str,
) -> Vec<AutocompleteOption> {
    let conn = app_state.guild_db();

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
    let quest_list_service = QuestListService::new(app_state.repositories.quest);
    let results = quest_list_service
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
pub async fn list_quests_for_select(app_state: &AppState) -> Vec<(String, i32)> {
    let service = QuestListService::new(app_state.repositories.quest);
    match service.list_for_select(app_state.guild_db(), 25).await {
        Ok(list) => list,
        Err(e) => {
            error!(error = %e, "クエスト一覧の取得に失敗しました");
            vec![]
        }
    }
}

/// クエストIDから名称を取得
pub async fn get_quest_name_by_id(app_state: &AppState, quest_id: i32) -> Option<String> {
    let service = QuestListService::new(app_state.repositories.quest);
    service
        .get_name_by_id(app_state.guild_db(), quest_id)
        .await
        .unwrap_or_default()
}
