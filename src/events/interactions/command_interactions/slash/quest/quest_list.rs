use crate::events::helpers::get_message_or_key_from_context;
use crate::facades::recruitment::quest_management_facade;
use crate::services::message::MessageTextId;
use crate::types::{AppError, PoiseContext, Result};
use poise::ChoiceParameter;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, ChoiceParameter)]
pub enum QuestFilterType {
    #[name = "All quests"]
    #[name = "全て"]
    All,
    #[name = "Enabled only"]
    #[name = "有効のみ"]
    EnabledOnly,
    #[name = "Disabled only"]
    #[name = "無効のみ"]
    DisabledOnly,
}

/// クエスト一覧を表示
///
/// クエストの一覧を表示します。有効/無効で絞り込むことができます。
#[poise::command(
    slash_command,
    guild_only,
    ephemeral = true,
    rename = "quest_list",
    name_localized("ja", "クエスト一覧"),
    description_localized("ja", "クエストの一覧を表示します。（gbf_bot_controlロール必須）")
)]
pub async fn quest_list(
    ctx: PoiseContext<'_>,
    #[name_localized("ja", "絞り込み")]
    #[description = "Filter type"]
    #[description_localized("ja", "有効/無効で絞り込み")]
    filter: Option<QuestFilterType>,
) -> Result<()> {
    ctx.defer_ephemeral().await?;

    let guild_id = match ctx.guild_id() {
        Some(id) => id.get() as i64,
        None => {
            let message = get_message_or_key_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                MessageTextId::ErrorsGuildOnly,
                HashMap::new(),
            )
            .await;
            return Err(AppError::Business { message });
        }
    };

    let filter_type = filter.unwrap_or(QuestFilterType::All);
    let filter = match filter_type {
        QuestFilterType::All => quest_management_facade::QuestListFilter::All,
        QuestFilterType::EnabledOnly => quest_management_facade::QuestListFilter::EnabledOnly,
        QuestFilterType::DisabledOnly => quest_management_facade::QuestListFilter::DisabledOnly,
    };
    let list_result =
        quest_management_facade::list_quests(&ctx.data().app_state, guild_id, filter).await?;

    let message = match list_result {
        quest_management_facade::QuestListResult::All(all_quests) => {
            // クエスト名と有効/無効のリストを作成
            let title = get_message_or_key_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                MessageTextId::QuestListTitleAll,
                HashMap::new(),
            )
            .await;
            let status_enabled = get_message_or_key_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                MessageTextId::RecruitmentScheduleListStatusEnabled,
                HashMap::new(),
            )
            .await;
            let status_disabled = get_message_or_key_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                MessageTextId::RecruitmentScheduleListStatusDisabled,
                HashMap::new(),
            )
            .await;
            let mut lines = vec![title, String::new()];

            for quest in all_quests.iter().take(100) {
                let status = if quest.is_enabled {
                    status_enabled.as_str()
                } else {
                    status_disabled.as_str()
                };
                lines.push(format!("{} {}", status, quest.name));
            }

            if all_quests.len() > 100 {
                let mut params = HashMap::new();
                params.insert("count".to_string(), (all_quests.len() - 100).to_string());
                let more_count = get_message_or_key_from_context(
                    &ctx,
                    ctx.data().app_state.message_service(),
                    MessageTextId::QuestListMoreCount,
                    params,
                )
                .await;
                lines.push(more_count);
            }

            lines.join("\n")
        }
        quest_management_facade::QuestListResult::Enabled(results) => {
            let title = get_message_or_key_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                MessageTextId::QuestListTitleEnabled,
                HashMap::new(),
            )
            .await;
            let status_enabled = get_message_or_key_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                MessageTextId::RecruitmentScheduleListStatusEnabled,
                HashMap::new(),
            )
            .await;
            let mut lines = vec![title, String::new()];

            for result in results.iter().take(100) {
                lines.push(format!("{status_enabled} {result}"));
            }

            if results.len() > 100 {
                let mut params = HashMap::new();
                params.insert("count".to_string(), (results.len() - 100).to_string());
                let more_count = get_message_or_key_from_context(
                    &ctx,
                    ctx.data().app_state.message_service(),
                    MessageTextId::QuestListMoreCount,
                    params,
                )
                .await;
                lines.push(more_count);
            }

            if results.is_empty() {
                let empty_message = get_message_or_key_from_context(
                    &ctx,
                    ctx.data().app_state.message_service(),
                    MessageTextId::QuestListEmptyEnabled,
                    HashMap::new(),
                )
                .await;
                lines.push(empty_message);
            }

            lines.join("\n")
        }
        quest_management_facade::QuestListResult::Disabled(results) => {
            let title = get_message_or_key_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                MessageTextId::QuestListTitleDisabled,
                HashMap::new(),
            )
            .await;
            let status_disabled = get_message_or_key_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                MessageTextId::RecruitmentScheduleListStatusDisabled,
                HashMap::new(),
            )
            .await;
            let mut lines = vec![title, String::new()];

            for result in results.iter().take(100) {
                lines.push(format!("{status_disabled} {result}"));
            }

            if results.len() > 100 {
                let mut params = HashMap::new();
                params.insert("count".to_string(), (results.len() - 100).to_string());
                let more_count = get_message_or_key_from_context(
                    &ctx,
                    ctx.data().app_state.message_service(),
                    MessageTextId::QuestListMoreCount,
                    params,
                )
                .await;
                lines.push(more_count);
            }

            if results.is_empty() {
                let empty_message = get_message_or_key_from_context(
                    &ctx,
                    ctx.data().app_state.message_service(),
                    MessageTextId::QuestListEmptyDisabled,
                    HashMap::new(),
                )
                .await;
                lines.push(empty_message);
            }

            lines.join("\n")
        }
    };

    ctx.say(message).await?;

    Ok(())
}
