use crate::facades::recruitment::battle_style_list;
use crate::facades::recruitment::quest_list;
use crate::facades::recruitment::recruitment_schedule_list;
use crate::facades::guild_settings::GuildSettingsFacade;
use crate::types::PoiseContext;
use poise::serenity_prelude::AutocompleteChoice;
use std::sync::Arc;

/// クエスト名の入力候補を取得
pub async fn quest_auto_complete(ctx: PoiseContext<'_>, partial: &str) -> Vec<AutocompleteChoice> {
    quest_list::search_quests_for_autocomplete(ctx, partial).await
}

/// 攻略方法の入力候補を取得
pub async fn battle_style_auto_complete(
    ctx: PoiseContext<'_>,
    _partial: &str,
) -> Vec<AutocompleteChoice> {
    battle_style_list::get_battle_styles_for_autocomplete(ctx).await
}

/// タイムゾーンの入力候補を取得
pub async fn timezone_auto_complete(
    ctx: PoiseContext<'_>,
    partial: &str,
) -> Vec<AutocompleteChoice> {
    // Facade 経由に統一（DB不要だが、インターフェース一貫性のため Facade を使用）
    let app_state = &ctx.data().app_state;
    let facade = GuildSettingsFacade::new(Arc::new(app_state.clone()));
    facade.get_timezones_for_autocomplete(partial)
}

/// 募集スケジュールの入力候補を取得
pub async fn recruitment_schedule_auto_complete(
    ctx: PoiseContext<'_>,
    _partial: &str,
) -> Vec<AutocompleteChoice> {
    // Facade経由で取得（Tx/RLSと整形はファサード／サービスへ移譲）
    recruitment_schedule_list::get_schedules_for_autocomplete(ctx).await
}

/// ロケールの入力候補を取得
pub async fn locale_auto_complete(
    _ctx: PoiseContext<'_>,
    partial: &str,
) -> Vec<AutocompleteChoice> {
    let locales = [("ja", "日本語"),
        ("en", "English")];

    locales
        .iter()
        .filter(|(code, name)| {
            code.contains(partial) || name.to_lowercase().contains(&partial.to_lowercase())
        })
        .map(|(code, name)| AutocompleteChoice::new(format!("{name} ({code})"), *code))
        .collect()
}
