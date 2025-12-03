use crate::repository::database::battle_style_repository::{
    BattleStyleRepository, SeaOrmBattleStyleRepository,
};
use crate::types::PoiseContext;
use poise::serenity_prelude::AutocompleteChoice;
use tracing::error;

/// 攻略方法の入力候補を取得するファサード
///
/// オートコンプリートで攻略方法を取得する際に使用する。
/// すべての攻略方法をリポジトリから取得し、AutocompleteChoiceに変換して返す。
pub async fn get_battle_styles_for_autocomplete(
    ctx: PoiseContext<'_>,
) -> Vec<AutocompleteChoice> {
    // AppStateからDB接続を取得してRepositoryを作成
    let db_conn = ctx.data().app_state.db().clone();
    let battle_style_repository = SeaOrmBattleStyleRepository::new(db_conn);

    // すべての攻略方法を取得
    let battle_styles = battle_style_repository
        .get_all()
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
