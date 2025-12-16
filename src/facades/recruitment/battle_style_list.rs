use crate::repository::database::battle_style_repository::{
    BattleStyleRepository, SeaOrmBattleStyleRepository,
};
use crate::types::PoiseContext;
use poise::serenity_prelude::AutocompleteChoice;
use sea_orm::DatabaseConnection;
use tracing::error;

/// 攻略方法の入力候補を取得するファサード
///
/// オートコンプリートで攻略方法を取得する際に使用する。
/// すべての攻略方法をリポジトリから取得し、AutocompleteChoiceに変換して返す。
pub async fn get_battle_styles_for_autocomplete(ctx: PoiseContext<'_>) -> Vec<AutocompleteChoice> {
    // AppStateからDB接続を取得してRepositoryを作成
    let db_conn = ctx.data().app_state.guild_db();
    let battle_style_repository = SeaOrmBattleStyleRepository::new();

    // すべての攻略方法を取得
    let battle_styles = battle_style_repository
        .get_all(db_conn)
        .await
        .unwrap_or_else(|e| {
            error!(error = %e, "攻略方法の取得に失敗しました");
            vec![]
        });

    // AutocompleteChoiceに変換
    battle_styles
        .into_iter()
        .map(|style| AutocompleteChoice::new(style.display_name, style.id))
        .collect()
}

/// セレクトメニュー用に攻略方法一覧を返す
pub async fn list_battle_styles_for_select(ctx: PoiseContext<'_>) -> Vec<(String, i32)> {
    let db_conn = ctx.data().app_state.guild_db();
    let repo = SeaOrmBattleStyleRepository::new();
    match repo.get_all(db_conn).await {
        Ok(list) => list.into_iter().map(|s| (s.display_name, s.id)).collect(),
        Err(e) => {
            error!(error = %e, "攻略方法一覧の取得に失敗しました");
            vec![]
        }
    }
}

/// セレクトメニュー用に攻略方法一覧を返す（DB直渡し版）
pub async fn list_battle_styles_for_select_with_db(db: &DatabaseConnection) -> Vec<(String, i32)> {
    let repo = SeaOrmBattleStyleRepository::new();
    match repo.get_all(db).await {
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
    let repo = SeaOrmBattleStyleRepository::new();
    match repo.get_by_id(db, battle_style_id).await {
        Ok(Some(model)) => Some(model.display_name),
        _ => None,
    }
}
