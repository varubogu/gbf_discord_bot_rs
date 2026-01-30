use crate::events::converters::to_autocomplete_choices;
use crate::facades::guild_settings::GuildSettingsFacade;
use crate::facades::recruitment::battle_style_list;
use crate::facades::recruitment::quest_list;
use crate::facades::recruitment::recruitment_schedule_list;
use crate::types::PoiseContext;
use poise::serenity_prelude::AutocompleteChoice;
use std::sync::Arc;

/// クエスト名の入力候補を取得
pub async fn quest_auto_complete(ctx: PoiseContext<'_>, partial: &str) -> Vec<AutocompleteChoice> {
    let quest_list = quest_list::search_quests_for_autocomplete(ctx, partial).await;
    to_autocomplete_choices(quest_list)
}

/// 攻略方法の入力候補を取得
pub async fn battle_style_auto_complete(
    ctx: PoiseContext<'_>,
    _partial: &str,
) -> Vec<AutocompleteChoice> {
    let options = battle_style_list::get_battle_styles_for_autocomplete(ctx).await;
    to_autocomplete_choices(options)
}

/// タイムゾーンの入力候補を取得
pub async fn timezone_auto_complete(
    ctx: PoiseContext<'_>,
    partial: &str,
) -> Vec<AutocompleteChoice> {
    // Facade 経由に統一（DB不要だが、インターフェース一貫性のため Facade を使用）
    let app_state = &ctx.data().app_state;
    let facade = GuildSettingsFacade::new(Arc::new(app_state.clone()));
    let options = facade.get_timezones_for_autocomplete(partial);
    to_autocomplete_choices(options)
}

/// 募集スケジュールの入力候補を取得
pub async fn recruitment_schedule_auto_complete(
    ctx: PoiseContext<'_>,
    _partial: &str,
) -> Vec<AutocompleteChoice> {
    // Facade経由で取得（Tx/RLSと整形はファサード／サービスへ移譲）
    let options = recruitment_schedule_list::get_schedules_for_autocomplete(ctx).await;
    to_autocomplete_choices(options)
}

/// ロケールの入力候補を取得
pub async fn locale_auto_complete(
    _ctx: PoiseContext<'_>,
    partial: &str,
) -> Vec<AutocompleteChoice> {
    let locales = [("ja", "日本語"), ("en", "English")];

    locales
        .iter()
        .filter(|(code, name)| {
            code.contains(partial) || name.to_lowercase().contains(&partial.to_lowercase())
        })
        .map(|(code, name)| AutocompleteChoice::new(format!("{name} ({code})"), *code))
        .collect()
}
