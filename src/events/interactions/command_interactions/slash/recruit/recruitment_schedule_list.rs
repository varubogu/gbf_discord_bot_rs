use crate::events::helpers::get_message_or_key_from_context;
use crate::facades::recruitment::recruitment_schedule_facade::{
    RecruitmentScheduleFacade, ScheduleListItem,
};
use crate::services::message::MessageTextId;
use crate::types::{AppError, PoiseContext, Result};
use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter};
use std::collections::HashMap;
use tracing::info;

/// マルチ募集スケジュール一覧を表示
///
/// 登録されているマルチ募集スケジュールを表示します。
#[poise::command(
    slash_command,
    rename = "recruitment-schedule-list",
    guild_only,
    ephemeral = true,
    name_localized("ja", "定期募集一覧"),
    description_localized("ja", "登録されているマルチ募集スケジュールを表示します")
)]
pub async fn recruitment_schedule_list(
    ctx: PoiseContext<'_>,
    #[name_localized("ja", "全員表示")]
    #[description = "Show all schedules in this server (default: true)"]
    #[description_localized("ja", "このサーバーの全員のスケジュールを表示（デフォルト: true）")]
    show_all: Option<bool>,
) -> Result<()> {
    let guild_id = match ctx.guild_id() {
        Some(id) => id,
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

    let user_id = ctx.author().id;
    let show_all = show_all.unwrap_or(true);

    info!(
        guild_id = guild_id.get(),
        user_id = user_id.get(),
        show_all = show_all,
        "定期募集一覧コマンドが実行されました"
    );

    ctx.defer_ephemeral().await?;

    let app_state = &ctx.data().app_state;
    // ✅ Facadeを利用して一覧を取得（トランザクションやタイムゾーン取得はFacade内部）
    let facade = RecruitmentScheduleFacade::new(std::sync::Arc::new(app_state.clone()));
    let schedules: Vec<ScheduleListItem> = facade
        .list_recruitment_schedules(guild_id.get() as i64, user_id.get() as i64, show_all)
        .await?;
    let title = get_message_or_key_from_context(
        &ctx,
        ctx.data().app_state.message_service(),
        if show_all {
            MessageTextId::RecruitmentScheduleListTitleAll
        } else {
            MessageTextId::RecruitmentScheduleListTitleSelf
        },
        HashMap::new(),
    )
    .await;

    if schedules.is_empty() {
        let message_id = if show_all {
            MessageTextId::RecruitmentScheduleListEmptyAll
        } else {
            MessageTextId::RecruitmentScheduleListEmptySelf
        };
        let empty_description = get_message_or_key_from_context(
            &ctx,
            ctx.data().app_state.message_service(),
            message_id,
            HashMap::new(),
        )
        .await;

        let embed = CreateEmbed::default()
            .title(&title)
            .description(empty_description)
            .color(0xffaa00);

        ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
            .await?;
        return Ok(());
    }

    // 最大10件に制限
    let display_count = schedules.len().min(10);
    let total_count = schedules.len();

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

    let mut description = String::new();
    for (i, item) in schedules.iter().take(display_count).enumerate() {
        let status = if item.is_enabled {
            status_enabled.as_str()
        } else {
            status_disabled.as_str()
        };

        let dismissal_display = if let Some(dismissal) = &item.dismissal_times {
            let mut params = HashMap::new();
            params.insert("dismissal".to_string(), dismissal.to_string());
            get_message_or_key_from_context(
                &ctx,
                ctx.data().app_state.message_service(),
                MessageTextId::RecruitmentScheduleListDismissalPrefix,
                params,
            )
            .await
        } else {
            String::new()
        };

        description.push_str(&format!(
            "{}. **{}** (ID: {}) {}\n\
                クエスト: {}\n\
                曜日: {} ({})\n\
                開始: {:02}:{:02}\n\
                募集: {}日前の{}{}\n\
                作成者: <@{}>\n\n",
            i + 1,
            item.name,
            item.id,
            status,
            item.quest_name,
            item.days_str,
            item.timezone,
            item.quest_start_hour,
            item.quest_start_minute,
            item.recruit_day_offset,
            item.recruit_time_str,
            dismissal_display,
            item.created_by
        ));
    }

    if total_count > display_count {
        let mut params = HashMap::new();
        params.insert(
            "count".to_string(),
            (total_count - display_count).to_string(),
        );
        let more_count = get_message_or_key_from_context(
            &ctx,
            ctx.data().app_state.message_service(),
            MessageTextId::RecruitmentScheduleListMoreCount,
            params,
        )
        .await;
        description.push_str(&more_count);
    }

    let mut footer_params = HashMap::new();
    footer_params.insert("total_count".to_string(), total_count.to_string());
    footer_params.insert("display_count".to_string(), display_count.to_string());
    let footer = get_message_or_key_from_context(
        &ctx,
        ctx.data().app_state.message_service(),
        MessageTextId::RecruitmentScheduleListFooter,
        footer_params,
    )
    .await;

    let embed = CreateEmbed::default()
        .title(title)
        .description(description)
        .color(0x00aaff)
        .footer(CreateEmbedFooter::new(footer));

    ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
        .await?;

    Ok(())
}

// 表示用の曜日フォーマットはサービス層で実施（ScheduleQueryService::format_days）
