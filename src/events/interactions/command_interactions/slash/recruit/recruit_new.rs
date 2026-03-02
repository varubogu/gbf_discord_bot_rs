use crate::errors::RecruitmentError;
use crate::events::converters::to_create_embed;
use crate::facades::recruitment;
use crate::gateway::PoiseDiscordGateway;
use crate::types::{AppError, PoiseContext, Result};
use crate::utils::discord_helper;
use poise::serenity_prelude::ReactionType;
use poise::serenity_prelude::all::MessageId;
use std::sync::Arc;

use super::super::autocomplete::{battle_style_auto_complete, quest_auto_complete};

#[poise::command(
    slash_command,
    name_localized("ja", "マルチバトル募集"),
    description_localized("ja", "マルチバトル募集を作成します")
)]
pub async fn recruit_new(
    ctx: PoiseContext<'_>,

    #[autocomplete = "quest_auto_complete"]
    #[name_localized("ja", "クエスト名")]
    #[description = "quest name or alias"]
    #[description_localized("ja", "クエスト名またはクエスト別名")]
    quest: String,

    #[name_localized("ja", "クエスト出発日時")]
    #[description = "Quest departure date and time"]
    #[description_localized("ja", "クエスト出発日時")]
    event_date: String,

    #[autocomplete = "battle_style_auto_complete"]
    #[name_localized("ja", "マルチ攻略方法")]
    #[description = "battle style"]
    #[description_localized("ja", "マルチ攻略方法（未指定の場合はクエストのデフォルト値を使用）")]
    battle_style: Option<i32>,

    #[name_localized("ja", "解散時刻")]
    #[description = "dismissal times (comma-separated, max 3)"]
    #[description_localized("ja", "解散時刻（カンマ区切り、最大3つ。例: 1時間前, 21:00, 2日前）")]
    dismissal_times: Option<String>,
) -> Result<()> {
    ctx.defer().await?;

    // ギルドIDを取得
    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| AppError::from(RecruitmentError::GuildOnly))?;

    let app_state = &ctx.data().app_state;

    // Gateway作成
    let gateway = PoiseDiscordGateway::new(Arc::clone(&ctx.serenity_context().http));

    // Facade呼び出し（データ作成とDB保存、message_id=0で仮保存）
    let result = recruitment::new_recruit::new_recruitment(
        app_state,
        &gateway,
        guild_id.get(),
        ctx.channel_id().get(),
        &quest,
        battle_style,
        Some(event_date),
        false, // リアクション版
        dismissal_times,
        ctx.author().id.get(),
    )
    .await?;

    // メッセージ送信（events層で実行）
    let reply = poise::CreateReply::default()
        .content(&result.message_content)
        .embed(to_create_embed(&result.embed_content));

    let message = ctx.send(reply).await?;
    let message_id = message.message().await?.id.get();

    // message_idをDBに更新
    recruitment::new_recruit::update_message_id(
        app_state,
        guild_id.get(),
        result.recruitment_id,
        message_id,
    )
    .await?;

    // リアクション追加（UI層ヘルパー経由）
    let reactions: Vec<ReactionType> = result
        .reaction_emojis
        .iter()
        .map(|emoji| ReactionType::Unicode(emoji.clone()))
        .collect();

    discord_helper::add_reactions(
        ctx.serenity_context(),
        ctx.channel_id(),
        MessageId::new(message_id),
        &reactions,
    )
    .await
    .map_err(|e| {
        AppError::from(RecruitmentError::Message {
            message: format!("リアクション追加に失敗しました: {e}"),
        })
    })?;

    Ok(())
}
