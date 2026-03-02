use crate::events::permission::check_bot_control_role;
use crate::facades::recruitment::quest_management_facade;
use crate::types::{PoiseContext, Result};
use poise::serenity_prelude::AutocompleteChoice;
use tracing::error;

/// クエスト名の入力候補を取得（無効化されていないクエストのみ）
async fn quest_name_autocomplete(ctx: PoiseContext<'_>, partial: &str) -> Vec<AutocompleteChoice> {
    let guild_id = match ctx.guild_id() {
        Some(id) => id.get() as i64,
        None => {
            error!("ギルドIDが取得できませんでした");
            return vec![];
        }
    };

    let results = quest_management_facade::search_enabled_quests_for_autocomplete(
        &ctx.data().app_state,
        guild_id,
        partial,
    )
    .await;

    results
        .into_iter()
        .map(|r| AutocompleteChoice::new(r.display_name, r.quest_name))
        .collect()
}

/// クエストを無効化
///
/// 指定したクエストを無効化します（最大6つまで）。無効化されたクエストは新規募集時のオートコンプリートに表示されなくなります。
#[poise::command(
    slash_command,
    guild_only,
    check = "check_bot_control_role",
    ephemeral = true,
    rename = "quest_disable",
    name_localized("ja", "クエスト無効化"),
    description_localized(
        "ja",
        "クエストを無効化します（最大6つ）。（gbf_bot_controlロール必須）"
    )
)]
pub async fn quest_disable(
    ctx: PoiseContext<'_>,
    #[autocomplete = "quest_name_autocomplete"]
    #[name_localized("ja", "クエスト名1")]
    #[description = "Quest name to disable"]
    #[description_localized("ja", "無効化するクエスト名")]
    quest_name_1: String,

    #[autocomplete = "quest_name_autocomplete"]
    #[name_localized("ja", "クエスト名2")]
    #[description = "Quest name to disable"]
    #[description_localized("ja", "無効化するクエスト名")]
    quest_name_2: Option<String>,

    #[autocomplete = "quest_name_autocomplete"]
    #[name_localized("ja", "クエスト名3")]
    #[description = "Quest name to disable"]
    #[description_localized("ja", "無効化するクエスト名")]
    quest_name_3: Option<String>,

    #[autocomplete = "quest_name_autocomplete"]
    #[name_localized("ja", "クエスト名4")]
    #[description = "Quest name to disable"]
    #[description_localized("ja", "無効化するクエスト名")]
    quest_name_4: Option<String>,

    #[autocomplete = "quest_name_autocomplete"]
    #[name_localized("ja", "クエスト名5")]
    #[description = "Quest name to disable"]
    #[description_localized("ja", "無効化するクエスト名")]
    quest_name_5: Option<String>,

    #[autocomplete = "quest_name_autocomplete"]
    #[name_localized("ja", "クエスト名6")]
    #[description = "Quest name to disable"]
    #[description_localized("ja", "無効化するクエスト名")]
    quest_name_6: Option<String>,
) -> Result<()> {
    ctx.defer_ephemeral().await?;

    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| crate::types::AppError::Business {
            message: "このコマンドはサーバー内でのみ使用できます。".to_string(),
        })?
        .get() as i64;

    let mut quest_names = vec![quest_name_1];
    if let Some(name) = quest_name_2 {
        quest_names.push(name);
    }
    if let Some(name) = quest_name_3 {
        quest_names.push(name);
    }
    if let Some(name) = quest_name_4 {
        quest_names.push(name);
    }
    if let Some(name) = quest_name_5 {
        quest_names.push(name);
    }
    if let Some(name) = quest_name_6 {
        quest_names.push(name);
    }

    let result = quest_management_facade::change_quest_state(
        &ctx.data().app_state,
        guild_id,
        quest_names,
        quest_management_facade::QuestStateChangeAction::Disable,
    )
    .await?;
    let success_count = result.changed_count;
    let already_disabled = result.already_in_target_state;
    let not_found = result.not_found;

    // 結果メッセージを作成
    let mut message_parts = Vec::new();

    if success_count > 0 {
        message_parts.push(format!(
            "✅ {success_count}件のクエストを無効化しました。\n新規募集時のオートコンプリートに表示されなくなります。"
        ));
    }

    if !already_disabled.is_empty() {
        message_parts.push(format!(
            "\n⚠️ 既に無効化されています: {}",
            already_disabled.join(", ")
        ));
    }

    if !not_found.is_empty() {
        message_parts.push(format!(
            "\n❌ 見つかりませんでした: {}",
            not_found.join(", ")
        ));
    }

    if message_parts.is_empty() {
        message_parts.push("処理するクエストがありませんでした。".to_string());
    }

    ctx.say(message_parts.join("\n")).await?;

    Ok(())
}
