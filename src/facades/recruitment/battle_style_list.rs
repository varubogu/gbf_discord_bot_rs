use crate::services::recruitment::battle_style_query_service::BattleStyleQueryService;
use crate::types::PoiseContext;
use crate::types::discord::AutocompleteOption;
use sea_orm::DatabaseConnection;
use tracing::error;

/// 攻略方法の入力候補を取得するファサード
///
/// オートコンプリートで攻略方法を取得する際に使用する。
/// すべての攻略方法をサービスから取得し、AutocompleteOptionに変換して返す。
pub async fn get_battle_styles_for_autocomplete(ctx: PoiseContext<'_>) -> Vec<AutocompleteOption> {
    // AppStateからDB接続を取得
    let db_conn = ctx.data().app_state.guild_db();
    let service = BattleStyleQueryService::new();

    // すべての攻略方法を取得
    let battle_styles = service
        .get_all_battle_styles(db_conn)
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

/// セレクトメニュー用に攻略方法一覧を返す（DB直渡し版）
pub async fn list_battle_styles_for_select_with_db(db: &DatabaseConnection) -> Vec<(String, i32)> {
    let service = BattleStyleQueryService::new();
    match service.get_all_battle_styles(db).await {
        Ok(list) => list.into_iter().map(|s| (s.display_name, s.id)).collect(),
        Err(e) => {
            error!(error = %e, "攻略方法一覧の取得に失敗しました");
            vec![]
        }
    }
}

/// 攻略方法IDから名称を取得（DB直渡し版）
pub async fn get_battle_style_name_by_id_with_db(
    db: &DatabaseConnection,
    battle_style_id: i32,
) -> Option<String> {
    let service = BattleStyleQueryService::new();
    match service.get_battle_style_by_id(db, battle_style_id).await {
        Ok(model) => Some(model.display_name),
        _ => None,
    }
}
