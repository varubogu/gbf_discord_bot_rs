use crate::events::helpers::resolve_guild_locale;
use crate::events::permission::resolve_bot_control_for_interaction;
use crate::facades::guild_settings::GuildSettingsFacade;
use crate::facades::recruitment::{battle_style_list, quest_list};
use crate::gateway::PoiseDiscordGateway;
use crate::services::message::MessageTextId;
use crate::types::discord::MessageData;
use crate::types::{AppError, PoiseData, Result};
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use poise::serenity_prelude::{
    ButtonStyle, ChannelId, ComponentInteraction, ComponentInteractionDataKind, Context,
    CreateActionRow, CreateButton, CreateInputText, CreateInteractionResponse,
    CreateInteractionResponseMessage, CreateModal, CreateSelectMenu, CreateSelectMenuKind,
    CreateSelectMenuOption, InputTextStyle,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct DraftKey {
    user_id: u64,
    channel_id: u64,
    message_id: u64,
}

#[derive(Debug, Clone, Default)]
struct RecruitChangeDraft {
    quest_name: Option<String>,
    battle_style_id: Option<i32>,
    battle_style_name: Option<String>,
    event_date: Option<DateTime<Utc>>,
}

lazy_static! {
    static ref RECRUIT_CHANGE_DRAFTS: RwLock<HashMap<DraftKey, RecruitChangeDraft>> =
        RwLock::new(HashMap::new());
}

const QUEST_NONE_VALUE: &str = "__none_quest__";
const STYLE_NONE_VALUE: &str = "__none_style__";

const ID_PREFIX_QUEST: &str = "recruit_change_quest";
const ID_PREFIX_STYLE: &str = "recruit_change_style";
const ID_PREFIX_OPEN_DATE_MODAL: &str = "recruit_change_open_date_modal";
const ID_PREFIX_CLEAR_DATE: &str = "recruit_change_clear_date";
const ID_PREFIX_APPLY: &str = "recruit_change_apply";

async fn get_message_or_fallback(
    data: &PoiseData,
    guild_id: Option<u64>,
    message_id: MessageTextId,
    params: HashMap<String, String>,
    locale: &str,
    fallback_text: &str,
) -> String {
    data.app_state
        .message_service()
        .get_message(
            data.app_state.guild_db(),
            message_id.as_str(),
            params,
            guild_id.map(|id| id as i64),
            Some(locale),
        )
        .await
        .unwrap_or_else(|_| fallback_text.to_string())
}

/// 募集変更関連のコンポーネントインタラクションを処理
pub async fn handle_recruit_change_interaction(
    ctx: &Context,
    interaction: &ComponentInteraction,
    data: &PoiseData,
) -> Result<()> {
    let custom_id = &interaction.data.custom_id;

    if custom_id.starts_with(&format!("{ID_PREFIX_QUEST}:")) {
        handle_quest_selection(ctx, interaction, data).await
    } else if custom_id.starts_with(&format!("{ID_PREFIX_STYLE}:")) {
        handle_battle_style_selection(ctx, interaction, data).await
    } else if custom_id.starts_with(&format!("{ID_PREFIX_OPEN_DATE_MODAL}:")) {
        handle_open_date_modal(ctx, interaction, data).await
    } else if custom_id.starts_with(&format!("{ID_PREFIX_CLEAR_DATE}:")) {
        handle_clear_date(ctx, interaction, data).await
    } else if custom_id.starts_with(&format!("{ID_PREFIX_APPLY}:")) {
        handle_apply_changes(ctx, interaction, data).await
    } else {
        Ok(())
    }
}

/// パネル表示用の本文とコンポーネントを作成
pub async fn build_panel_content_and_components(
    data: &PoiseData,
    user_id: u64,
    channel_id: u64,
    message_id: u64,
    guild_id: Option<u64>,
) -> Result<(String, Vec<CreateActionRow>)> {
    let locale = resolve_guild_locale(&data.app_state, guild_id.map(|id| id as i64)).await;
    let key = DraftKey {
        user_id,
        channel_id,
        message_id,
    };

    let draft = {
        let drafts = RECRUIT_CHANGE_DRAFTS.read().await;
        drafts.get(&key).cloned().unwrap_or_default()
    };

    let unchanged_quest = get_message_or_fallback(
        data,
        guild_id,
        MessageTextId::RecruitmentCommandChangePanelUnchanged,
        HashMap::new(),
        &locale,
        "変更しない",
    )
    .await;
    let unchanged_style = get_message_or_fallback(
        data,
        guild_id,
        MessageTextId::RecruitmentCommandChangePanelUnchanged,
        HashMap::new(),
        &locale,
        "変更しない",
    )
    .await;
    let apply_label = get_message_or_fallback(
        data,
        guild_id,
        MessageTextId::RecruitmentCommandChangeButtonApply,
        HashMap::new(),
        &locale,
        "適用",
    )
    .await;

    let quest_label = draft
        .quest_name
        .clone()
        .unwrap_or_else(|| unchanged_quest.clone());
    let style_label = draft
        .battle_style_name
        .clone()
        .unwrap_or_else(|| unchanged_style.clone());
    let date_label = format_event_date_label(data, guild_id, draft.event_date, &locale).await;

    let mut content_params = HashMap::new();
    content_params.insert("quest_label".to_string(), quest_label.clone());
    content_params.insert("style_label".to_string(), style_label.clone());
    content_params.insert("date_label".to_string(), date_label.clone());
    content_params.insert("apply_label".to_string(), apply_label.clone());
    let content = get_message_or_fallback(
        data,
        guild_id,
        MessageTextId::RecruitmentCommandChangePanelContent,
        content_params,
        &locale,
        &format!(
            "変更内容を選択・入力してください。\n\n\
             現在の入力値\n\
             - クエスト: {quest_label}\n\
             - 攻略方法: {style_label}\n\
             - 出発日時: {date_label}\n\n\
             `{apply_label}`を押すまで反映されません。"
        ),
    )
    .await;

    let quest_pairs = quest_list::list_quests_for_select(
        data.app_state.guild_db(),
        data.app_state.repositories.quest,
    )
    .await;

    let option_quest_unchanged = get_message_or_fallback(
        data,
        guild_id,
        MessageTextId::RecruitmentCommandChangeOptionQuestUnchanged,
        HashMap::new(),
        &locale,
        "クエスト：変更しない",
    )
    .await;
    let mut quest_options = vec![
        CreateSelectMenuOption::new(option_quest_unchanged, QUEST_NONE_VALUE)
            .default_selection(draft.quest_name.is_none()),
    ];
    quest_options.extend(quest_pairs.into_iter().take(24).map(|(name, id)| {
        let is_selected = draft
            .quest_name
            .as_ref()
            .map(|n| n == &name)
            .unwrap_or(false);
        CreateSelectMenuOption::new(name, id.to_string()).default_selection(is_selected)
    }));

    let style_pairs = battle_style_list::list_battle_styles_for_select(&data.app_state).await;
    let option_style_unchanged = get_message_or_fallback(
        data,
        guild_id,
        MessageTextId::RecruitmentCommandChangeOptionStyleUnchanged,
        HashMap::new(),
        &locale,
        "攻略方法：変更しない",
    )
    .await;
    let mut style_options = vec![
        CreateSelectMenuOption::new(option_style_unchanged, STYLE_NONE_VALUE)
            .default_selection(draft.battle_style_id.is_none()),
    ];
    style_options.extend(style_pairs.into_iter().take(24).map(|(name, id)| {
        let is_selected = draft.battle_style_id.map(|s| s == id).unwrap_or(false);
        CreateSelectMenuOption::new(name, id.to_string()).default_selection(is_selected)
    }));

    let quest_placeholder = get_message_or_fallback(
        data,
        guild_id,
        MessageTextId::RecruitmentCommandChangePlaceholderQuest,
        HashMap::new(),
        &locale,
        "クエストを選択",
    )
    .await;
    let quest_select = CreateSelectMenu::new(
        format!("{ID_PREFIX_QUEST}:{channel_id}:{message_id}"),
        CreateSelectMenuKind::String {
            options: quest_options,
        },
    )
    .placeholder(quest_placeholder);

    let style_placeholder = get_message_or_fallback(
        data,
        guild_id,
        MessageTextId::RecruitmentCommandChangePlaceholderStyle,
        HashMap::new(),
        &locale,
        "攻略方法を選択",
    )
    .await;
    let style_select = CreateSelectMenu::new(
        format!("{ID_PREFIX_STYLE}:{channel_id}:{message_id}"),
        CreateSelectMenuKind::String {
            options: style_options,
        },
    )
    .placeholder(style_placeholder);

    let open_date_label = get_message_or_fallback(
        data,
        guild_id,
        MessageTextId::RecruitmentCommandChangeButtonOpenDate,
        HashMap::new(),
        &locale,
        "出発日時を入力",
    )
    .await;
    let clear_date_label = get_message_or_fallback(
        data,
        guild_id,
        MessageTextId::RecruitmentCommandChangeButtonClearDate,
        HashMap::new(),
        &locale,
        "日時をクリア",
    )
    .await;

    let open_date_button = CreateButton::new(format!(
        "{ID_PREFIX_OPEN_DATE_MODAL}:{channel_id}:{message_id}"
    ))
    .style(ButtonStyle::Primary)
    .label(open_date_label);

    let clear_date_button =
        CreateButton::new(format!("{ID_PREFIX_CLEAR_DATE}:{channel_id}:{message_id}"))
            .style(ButtonStyle::Secondary)
            .label(clear_date_label);

    let apply_button = CreateButton::new(format!("{ID_PREFIX_APPLY}:{channel_id}:{message_id}"))
        .style(ButtonStyle::Success)
        .label(apply_label);

    Ok((
        content,
        vec![
            CreateActionRow::SelectMenu(quest_select),
            CreateActionRow::SelectMenu(style_select),
            CreateActionRow::Buttons(vec![open_date_button, clear_date_button, apply_button]),
        ],
    ))
}

async fn format_event_date_label(
    data: &PoiseData,
    guild_id: Option<u64>,
    event_date: Option<DateTime<Utc>>,
    locale: &str,
) -> String {
    let Some(event_date) = event_date else {
        return get_message_or_fallback(
            data,
            guild_id,
            MessageTextId::RecruitmentCommandChangePanelUnchanged,
            HashMap::new(),
            locale,
            "変更しない",
        )
        .await;
    };

    if let Some(guild_id) = guild_id {
        let timezone_facade = GuildSettingsFacade::new(Arc::new(data.app_state.clone()));
        match timezone_facade.get_timezone(guild_id as i64).await {
            Ok(timezone) => {
                return event_date
                    .with_timezone(&timezone)
                    .format("%Y-%m-%d %H:%M %Z")
                    .to_string();
            }
            Err(e) => {
                warn!(error = %e, guild_id = guild_id, "タイムゾーン取得に失敗したためUTC表示します");
            }
        }
    }

    event_date.format("%Y-%m-%d %H:%M UTC").to_string()
}

/// 日時ドラフトを更新
pub async fn set_event_date_draft(
    user_id: u64,
    channel_id: u64,
    message_id: u64,
    event_date: Option<DateTime<Utc>>,
) {
    let key = DraftKey {
        user_id,
        channel_id,
        message_id,
    };
    let mut drafts = RECRUIT_CHANGE_DRAFTS.write().await;
    let draft = drafts.entry(key).or_default();
    draft.event_date = event_date;
}

fn parse_target_ids(custom_id: &str, prefix: &str) -> Result<(u64, u64)> {
    let parts: Vec<&str> = custom_id.split(':').collect();
    if parts.len() != 3 || parts[0] != prefix {
        return Err(AppError::Generic("不正なカスタムIDです".to_string()));
    }

    let channel_id = parts[1]
        .parse::<u64>()
        .map_err(|_| AppError::Generic("チャンネルIDの解析に失敗しました".to_string()))?;
    let message_id = parts[2]
        .parse::<u64>()
        .map_err(|_| AppError::Generic("メッセージIDの解析に失敗しました".to_string()))?;

    Ok((channel_id, message_id))
}

async fn handle_quest_selection(
    ctx: &Context,
    interaction: &ComponentInteraction,
    data: &PoiseData,
) -> Result<()> {
    let (target_channel_id, target_message_id) =
        parse_target_ids(&interaction.data.custom_id, ID_PREFIX_QUEST)?;

    let selected_value = match &interaction.data.kind {
        ComponentInteractionDataKind::StringSelect { values } => values
            .first()
            .ok_or_else(|| AppError::Generic("クエストが選択されていません".to_string()))?,
        _ => {
            return Err(AppError::Generic(
                "予期しないコンポーネントタイプです".to_string(),
            ));
        }
    };

    let user_id = interaction.user.id.get();
    let key = DraftKey {
        user_id,
        channel_id: target_channel_id,
        message_id: target_message_id,
    };

    {
        let mut drafts = RECRUIT_CHANGE_DRAFTS.write().await;
        let draft = drafts.entry(key).or_default();

        if selected_value == QUEST_NONE_VALUE {
            draft.quest_name = None;
        } else {
            let quest_id: i32 = selected_value
                .parse()
                .map_err(|_| AppError::Generic("クエストIDの解析に失敗しました".to_string()))?;

            let quest_name = quest_list::get_quest_name_by_id(
                data.app_state.guild_db(),
                data.app_state.repositories.quest,
                quest_id,
            )
            .await
            .ok_or_else(|| AppError::Generic("クエストが見つかりません".to_string()))?;

            draft.quest_name = Some(quest_name);
        }
    }

    let (content, components) = build_panel_content_and_components(
        data,
        user_id,
        target_channel_id,
        target_message_id,
        interaction.guild_id.map(|id| id.get()),
    )
    .await?;

    interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new()
                    .content(content)
                    .components(components),
            ),
        )
        .await?;

    Ok(())
}

async fn handle_battle_style_selection(
    ctx: &Context,
    interaction: &ComponentInteraction,
    data: &PoiseData,
) -> Result<()> {
    let (target_channel_id, target_message_id) =
        parse_target_ids(&interaction.data.custom_id, ID_PREFIX_STYLE)?;

    let selected_value = match &interaction.data.kind {
        ComponentInteractionDataKind::StringSelect { values } => values
            .first()
            .ok_or_else(|| AppError::Generic("攻略方法が選択されていません".to_string()))?,
        _ => {
            return Err(AppError::Generic(
                "予期しないコンポーネントタイプです".to_string(),
            ));
        }
    };

    let user_id = interaction.user.id.get();
    let key = DraftKey {
        user_id,
        channel_id: target_channel_id,
        message_id: target_message_id,
    };

    {
        let mut drafts = RECRUIT_CHANGE_DRAFTS.write().await;
        let draft = drafts.entry(key).or_default();

        if selected_value == STYLE_NONE_VALUE {
            draft.battle_style_id = None;
            draft.battle_style_name = None;
        } else {
            let battle_style_id: i32 = selected_value
                .parse()
                .map_err(|_| AppError::Generic("攻略方法IDの解析に失敗しました".to_string()))?;
            let battle_style_name =
                battle_style_list::get_battle_style_name_by_id(&data.app_state, battle_style_id)
                    .await
                    .unwrap_or_else(|| format!("ID:{battle_style_id}"));

            draft.battle_style_id = Some(battle_style_id);
            draft.battle_style_name = Some(battle_style_name);
        }
    }

    let (content, components) = build_panel_content_and_components(
        data,
        user_id,
        target_channel_id,
        target_message_id,
        interaction.guild_id.map(|id| id.get()),
    )
    .await?;

    interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new()
                    .content(content)
                    .components(components),
            ),
        )
        .await?;

    Ok(())
}

async fn handle_open_date_modal(
    ctx: &Context,
    interaction: &ComponentInteraction,
    data: &PoiseData,
) -> Result<()> {
    let (target_channel_id, target_message_id) =
        parse_target_ids(&interaction.data.custom_id, ID_PREFIX_OPEN_DATE_MODAL)?;

    let custom_id = format!("recruit_change_date_modal:{target_channel_id}:{target_message_id}");

    let guild_id = interaction.guild_id.map(|id| id.get());
    let locale = resolve_guild_locale(&data.app_state, guild_id.map(|id| id as i64)).await;
    let modal_title = get_message_or_fallback(
        data,
        guild_id,
        MessageTextId::RecruitmentCommandChangeModalTitle,
        HashMap::new(),
        &locale,
        "出発日時変更",
    )
    .await;
    let modal_label = get_message_or_fallback(
        data,
        guild_id,
        MessageTextId::RecruitmentCommandChangeModalEventDateLabel,
        HashMap::new(),
        &locale,
        "出発日時",
    )
    .await;
    let modal_placeholder = get_message_or_fallback(
        data,
        guild_id,
        MessageTextId::RecruitmentCommandChangeModalEventDatePlaceholder,
        HashMap::new(),
        &locale,
        "例: 12/25 22:30",
    )
    .await;

    let modal =
        CreateModal::new(custom_id, modal_title).components(vec![CreateActionRow::InputText(
            CreateInputText::new(InputTextStyle::Short, modal_label, "event_date")
                .placeholder(modal_placeholder)
                .required(true),
        )]);

    interaction
        .create_response(&ctx.http, CreateInteractionResponse::Modal(modal))
        .await?;

    Ok(())
}

async fn handle_clear_date(
    ctx: &Context,
    interaction: &ComponentInteraction,
    data: &PoiseData,
) -> Result<()> {
    let (target_channel_id, target_message_id) =
        parse_target_ids(&interaction.data.custom_id, ID_PREFIX_CLEAR_DATE)?;

    let user_id = interaction.user.id.get();
    set_event_date_draft(user_id, target_channel_id, target_message_id, None).await;

    let (content, components) = build_panel_content_and_components(
        data,
        user_id,
        target_channel_id,
        target_message_id,
        interaction.guild_id.map(|id| id.get()),
    )
    .await?;

    interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new()
                    .content(content)
                    .components(components),
            ),
        )
        .await?;

    Ok(())
}

async fn handle_apply_changes(
    ctx: &Context,
    interaction: &ComponentInteraction,
    data: &PoiseData,
) -> Result<()> {
    let (target_channel_id, target_message_id) =
        parse_target_ids(&interaction.data.custom_id, ID_PREFIX_APPLY)?;

    let user_id = interaction.user.id.get();
    let interaction_guild_id = interaction
        .guild_id
        .ok_or_else(|| AppError::Generic("ギルドIDが取得できません".to_string()))?
        .get();
    let locale = resolve_guild_locale(&data.app_state, Some(interaction_guild_id as i64)).await;

    let key = DraftKey {
        user_id,
        channel_id: target_channel_id,
        message_id: target_message_id,
    };

    let draft = {
        let drafts = RECRUIT_CHANGE_DRAFTS.read().await;
        drafts.get(&key).cloned().unwrap_or_default()
    };

    if draft.quest_name.is_none() && draft.battle_style_id.is_none() && draft.event_date.is_none() {
        let (content, components) = build_panel_content_and_components(
            data,
            user_id,
            target_channel_id,
            target_message_id,
            Some(interaction_guild_id),
        )
        .await?;
        let no_changes_message = get_message_or_fallback(
            data,
            Some(interaction_guild_id),
            MessageTextId::RecruitmentCommandChangeNoChanges,
            HashMap::new(),
            &locale,
            "変更項目を少なくとも1つ指定してください。",
        )
        .await;

        interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new()
                        .content(format!("{content}\n\n{no_changes_message}"))
                        .components(components),
                ),
            )
            .await?;
        return Ok(());
    }

    interaction.defer(&ctx.http).await?;

    let target_message = ChannelId::new(target_channel_id)
        .message(&ctx.http, target_message_id)
        .await
        .map_err(|e| {
            error!(error = %e, channel_id = target_channel_id, message_id = target_message_id, "メッセージの取得に失敗しました");
            AppError::Generic("対象のメッセージが見つかりませんでした".to_string())
        })?;

    let target_guild_id = target_message
        .guild_id
        .map(|id| id.get())
        .unwrap_or(interaction_guild_id);

    if target_guild_id != interaction_guild_id {
        warn!(
            interaction_guild_id = interaction_guild_id,
            target_guild_id = target_guild_id,
            channel_id = target_channel_id,
            message_id = target_message_id,
            "募集変更のギルドIDが一致しません"
        );
    }

    let gateway = PoiseDiscordGateway::new(Arc::clone(&ctx.http));
    let message_data = MessageData::from(target_message);

    // 実行者情報を解決（events層でDiscordコンテキストから取得し、ドメイン値として渡す）
    let has_bot_control = resolve_bot_control_for_interaction(ctx, interaction).await;

    let result = crate::facades::recruitment::change::change_recruitment_information_internal(
        &data.app_state,
        &gateway,
        target_guild_id,
        &message_data,
        crate::facades::recruitment::change::RecruitmentChangeContent {
            quest: draft.quest_name,
            event_date: draft.event_date,
            battle_style_id: draft.battle_style_id,
        },
        user_id,
        has_bot_control,
    )
    .await;

    match result {
        Ok(_) => {
            let mut drafts = RECRUIT_CHANGE_DRAFTS.write().await;
            drafts.remove(&key);

            let success_message = get_message_or_fallback(
                data,
                Some(interaction_guild_id),
                MessageTextId::RecruitmentCommandChangeSuccess,
                HashMap::new(),
                &locale,
                "募集内容を更新しました。",
            )
            .await;

            interaction
                .edit_response(
                    &ctx.http,
                    poise::serenity_prelude::EditInteractionResponse::new()
                        .content(success_message)
                        .components(vec![]),
                )
                .await?;

            info!(
                user_id = user_id,
                guild_id = target_guild_id,
                channel_id = target_channel_id,
                message_id = target_message_id,
                "募集内容変更が完了しました"
            );
        }
        Err(AppError::Business { .. }) => {
            // 権限エラー等のビジネスエラーはロケール対応メッセージを表示
            let error_msg = get_message_or_fallback(
                data,
                Some(interaction_guild_id),
                MessageTextId::RecruitmentCommandChangePermissionDenied,
                HashMap::new(),
                &locale,
                "この募集の変更は作成者本人または gbf_bot_control ロールを持つ管理者のみ可能です。",
            )
            .await;

            interaction
                .edit_response(
                    &ctx.http,
                    poise::serenity_prelude::EditInteractionResponse::new()
                        .content(error_msg)
                        .components(vec![]),
                )
                .await?;
        }
        Err(e) => {
            error!(error = %e, "募集内容変更に失敗しました");

            let (content, components) = build_panel_content_and_components(
                data,
                user_id,
                target_channel_id,
                target_message_id,
                Some(interaction_guild_id),
            )
            .await?;
            let mut error_params = HashMap::new();
            error_params.insert("error_message".to_string(), e.user_message());
            let error_prefix = get_message_or_fallback(
                data,
                Some(interaction_guild_id),
                MessageTextId::CommonErrorPrefix,
                error_params,
                &locale,
                &format!("エラー: {}", e.user_message()),
            )
            .await;

            interaction
                .edit_response(
                    &ctx.http,
                    poise::serenity_prelude::EditInteractionResponse::new()
                        .content(format!("{error_prefix}\n\n{content}"))
                        .components(components),
                )
                .await?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::parse_target_ids;

    #[test]
    fn parse_target_ids_works() {
        let (channel_id, message_id) =
            parse_target_ids("recruit_change_apply:10:20", "recruit_change_apply")
                .expect("custom_id は解析できるべき");
        assert_eq!(channel_id, 10);
        assert_eq!(message_id, 20);
    }

    #[test]
    fn parse_target_ids_rejects_invalid_prefix() {
        let err = parse_target_ids("recruit_change_style:10:20", "recruit_change_apply")
            .expect_err("prefix不一致は失敗するべき");
        assert!(err.user_message().contains("不正なカスタムID"));
    }
}
