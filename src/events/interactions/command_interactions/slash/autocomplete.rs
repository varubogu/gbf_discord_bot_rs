use crate::facades::recruitment::battle_style_list;
use crate::facades::recruitment::quest_list;
use crate::types::PoiseContext;
use futures::Stream;
use poise::serenity_prelude::AutocompleteChoice;

/// クエスト名の入力候補を取得
pub async fn quest_auto_complete<'a>(
    ctx: PoiseContext<'_>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    quest_list::search_quests_for_autocomplete(ctx, partial).await
}

/// 攻略方法の入力候補を取得
pub async fn battle_style_auto_complete(
    ctx: PoiseContext<'_>,
    _partial: &str,
) -> Vec<AutocompleteChoice> {
    battle_style_list::get_battle_styles_for_autocomplete(ctx).await
}
