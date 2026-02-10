use crate::events::converters::to_autocomplete_choices;
use crate::facades::guild_settings::GuildSettingsFacade;
use crate::facades::recruitment::battle_style_list;
use crate::facades::recruitment::quest_list;
use crate::facades::recruitment::recruitment_schedule_list;
use crate::types::PoiseContext;
use poise::serenity_prelude::AutocompleteChoice;
use std::sync::Arc;
use tracing::error;

/// クエスト名の入力候補を取得
pub async fn quest_auto_complete(ctx: PoiseContext<'_>, partial: &str) -> Vec<AutocompleteChoice> {
    // events層でPoiseContextから値を抽出
    let guild_id = match ctx.guild_id() {
        Some(id) => id.get() as i64,
        None => {
            error!("ギルドIDが取得できませんでした");
            return vec![];
        }
    };
    let app_state = &ctx.data().app_state;
    let conn = app_state.guild_db();
    let quest_repository = app_state.repositories.quest;

    // facade層にはpoise依存のない値を渡す
    let quest_list =
        quest_list::search_quests_for_autocomplete(conn, &quest_repository, guild_id, partial)
            .await;
    to_autocomplete_choices(quest_list)
}

/// 攻略方法の入力候補を取得
pub async fn battle_style_auto_complete(
    ctx: PoiseContext<'_>,
    _partial: &str,
) -> Vec<AutocompleteChoice> {
    let app_state = &ctx.data().app_state;

    // facade層にはpoise依存のない値を渡す
    let options = battle_style_list::get_battle_styles_for_autocomplete(app_state).await;
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
    // events層でPoiseContextから値を抽出
    let guild_id = match ctx.guild_id() {
        Some(id) => id.get() as i64,
        None => {
            error!("ギルドIDが取得できませんでした");
            return vec![];
        }
    };
    let user_id = ctx.author().id.get() as i64;
    let app_state = &ctx.data().app_state;

    // facade層にはpoise依存のない値を渡す
    let options =
        recruitment_schedule_list::get_schedules_for_autocomplete(app_state, guild_id, user_id)
            .await;
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
