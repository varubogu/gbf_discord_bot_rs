use crate::events::helpers::{get_message_from_context, get_message_or_fallback_from_context};
use crate::facades::recruitment::recruit_list::list_active_recruitments;
use crate::services::message::MessageTextId;
use crate::types::{PoiseContext, Result};
use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter};
use std::collections::HashMap;
use tracing::info;

/// 現在募集中のマルチバトル一覧を表示
///
/// このサーバーで現在募集中のマルチバトルをメッセージリンク付きで一覧表示します。
#[poise::command(
    slash_command,
    rename = "recruit-list",
    guild_only,
    ephemeral = true,
    name_localized("ja", "募集一覧"),
    description_localized("ja", "現在募集中のマルチバトル一覧を表示します")
)]
pub async fn recruit_list(ctx: PoiseContext<'_>) -> Result<()> {
    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| crate::types::AppError::Business {
            message: "このコマンドはサーバー内でのみ使用できます".to_string(),
        })?;

    info!(
        guild_id = guild_id.get(),
        user_id = ctx.author().id.get(),
        "募集一覧コマンドが実行されました"
    );

    ctx.defer_ephemeral().await?;

    let app_state = &ctx.data().app_state;
    let result = list_active_recruitments(app_state, guild_id.get() as i64).await?;

    let title = get_message_or_fallback_from_context(
        &ctx,
        ctx.data().app_state.message_service(),
        MessageTextId::RecruitmentListTitle,
        HashMap::new(),
        "現在の募集一覧",
    )
    .await;

    // 募集なし
    if result.items.is_empty() {
        let empty_description = get_message_or_fallback_from_context(
            &ctx,
            ctx.data().app_state.message_service(),
            MessageTextId::RecruitmentListEmpty,
            HashMap::new(),
            "現在募集中のマルチバトルはありません。",
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

    // 日時フォーマット文字列を取得（ロケール対応）
    let date_format = get_message_or_fallback_from_context(
        &ctx,
        ctx.data().app_state.message_service(),
        MessageTextId::RecruitmentDisplayDateFormat,
        HashMap::new(),
        "%m/%d %H:%M %Z",
    )
    .await;

    // リンクテキストを取得
    let link_text = get_message_or_fallback_from_context(
        &ctx,
        ctx.data().app_state.message_service(),
        MessageTextId::RecruitmentListLinkText,
        HashMap::new(),
        "→ 募集メッセージ",
    )
    .await;

    // 最大25件に制限（embed description の文字数制限を考慮）
    let display_count = result.items.len().min(25);
    let total_count = result.items.len();

    let mut description = String::new();
    for (i, item) in result.items.iter().take(display_count).enumerate() {
        // UTC → ギルドタイムゾーンへ変換して表示
        let local_dt = item.quest_start_at.with_timezone(&result.timezone);
        let formatted_dt = local_dt.format(&date_format).to_string();

        // Discord メッセージリンク（Markdown ハイパーリンク形式）
        let message_url = format!(
            "https://discord.com/channels/{}/{}/{}",
            guild_id.get(),
            item.channel_id,
            item.message_id,
        );

        description.push_str(&format!(
            "{}. **{}**\n出発: {} [{}]({})\n\n",
            i + 1,
            item.quest_name,
            formatted_dt,
            link_text,
            message_url,
        ));
    }

    // 25件超の場合に末尾へ追記
    if total_count > display_count {
        let mut params = HashMap::new();
        params.insert(
            "count".to_string(),
            (total_count - display_count).to_string(),
        );
        let more_count = get_message_from_context(
            &ctx,
            ctx.data().app_state.message_service(),
            MessageTextId::RecruitmentListMoreCount,
            params,
        )
        .await
        .unwrap_or_else(|_| format!("*...他 {} 件の募集があります*", total_count - display_count));
        description.push_str(&more_count);
    }

    // フッター
    let mut footer_params = HashMap::new();
    footer_params.insert("total_count".to_string(), total_count.to_string());
    footer_params.insert("display_count".to_string(), display_count.to_string());
    let footer = get_message_from_context(
        &ctx,
        ctx.data().app_state.message_service(),
        MessageTextId::RecruitmentListFooter,
        footer_params,
    )
    .await
    .unwrap_or_else(|_| format!("全 {total_count} 件の募集（{display_count}件表示）"));

    let embed = CreateEmbed::default()
        .title(title)
        .description(description)
        .color(0x00aaff)
        .footer(CreateEmbedFooter::new(footer));

    ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
        .await?;

    Ok(())
}
