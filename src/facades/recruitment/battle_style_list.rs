use crate::services::recruitment::battle_style_query_service::BattleStyleQueryService;
use crate::types::AppState;
use crate::types::discord::AutocompleteOption;
use tracing::error;

/// 攻略方法の入力候補を取得するファサード
///
/// オートコンプリートで攻略方法を取得する際に使用する。
/// すべての攻略方法をサービスから取得し、AutocompleteOptionに変換して返す。
///
/// # 引数
/// * `app_state` - アプリケーション状態
pub async fn get_battle_styles_for_autocomplete(app_state: &AppState) -> Vec<AutocompleteOption> {
    let service = BattleStyleQueryService::new(app_state.repositories.battle_style);
    let conn = app_state.guild_db();

    // すべての攻略方法を取得
    let battle_styles = service
        .get_all_battle_styles(conn)
        .await
        .unwrap_or_else(|e| {
            error!(error = %e, "攻略方法の取得に失敗しました");
            vec![]
        });

    // AutocompleteOptionに変換
    battle_styles
        .into_iter()
        .map(|style| AutocompleteOption::new(style.display_name, style.id.to_string()))
        .collect()
}

/// セレクトメニュー用に攻略方法一覧を返す
pub async fn list_battle_styles_for_select(app_state: &AppState) -> Vec<(String, i32)> {
    let service = BattleStyleQueryService::new(app_state.repositories.battle_style);
    let db = app_state.guild_db();
    match service.get_all_battle_styles(db).await {
        Ok(list) => list.into_iter().map(|s| (s.display_name, s.id)).collect(),
        Err(e) => {
            error!(error = %e, "攻略方法一覧の取得に失敗しました");
            vec![]
        }
    }
}

/// 攻略方法IDから名称を取得
pub async fn get_battle_style_name_by_id(
    app_state: &AppState,
    battle_style_id: i32,
) -> Option<String> {
    let service = BattleStyleQueryService::new(app_state.repositories.battle_style);
    let db = app_state.guild_db();
    match service.get_battle_style_by_id(db, battle_style_id).await {
        Ok(model) => Some(model.display_name),
        _ => None,
    }
}
