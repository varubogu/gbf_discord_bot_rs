use crate::facades::battle_recruitment::BattleRecruitmentFacade;
use crate::types::battle_type::BattleType;
use crate::types::{PoiseContext, Result, DiscordOperation, DiscordOperationResult, DiscordOperationError};
use futures::Stream;
use poise::serenity_prelude::{Message, ChannelId, CreateMessage, CreateEmbed, ReactionType, Http};
use std::pin::Pin;
use std::future::Future;
use std::sync::Arc;

#[poise::command(
    slash_command,
    name_localized("ja", "募集"),
    description_localized("ja", "バトル募集を作成します")
)]
pub async fn recruit(
    ctx: PoiseContext<'_>,

    #[description = "quest name or alias"]
    #[description_localized("ja", "クエスト名またはクエスト別名")]
    #[autocomplete = "quest_auto_complete"]
    quest: String,

    #[description = "Quest departure date and time"]
    #[description_localized("ja", "クエスト出発日時")]
    event_date: String,
    // Temporarily removing BattleType parameter until traits are implemented
    // #[description = "Quest Combat Style"]
    // #[description_localized("ja", "クエストの戦闘スタイル")]
    // battle_type: Option<BattleType>,
) -> Result<()> {
    ctx.defer().await?;

    let battle_type = BattleType::Default;

    // let _event_datetime = RecruitmentService::parse_event_date(&event_date).await?;

    // guild_idとchannel_idを取得
    let guild_id = ctx.guild_id().map(|id| id.get()).unwrap_or(0);
    let channel_id = ctx.channel_id().get();

    // Create BattleRecruitmentFacade using AppState (Rustらしいパターン)
    let app_state = &ctx.data().app_state;
    let facade = BattleRecruitmentFacade::new(app_state);

    // Discord操作用のクロージャを作成
    let discord_http = ctx.serenity_context().http.clone();
    let mut discord_operation = |operation: DiscordOperation| -> Pin<Box<dyn Future<Output=Result<DiscordOperationResult>> + Send>> {
        let http = discord_http.clone();
        Box::pin(async move {
            match operation {
                DiscordOperation::SendMessage { channel_id, content, embed } => {
                    send_message_operation(http, channel_id, content, embed).await
                },
                DiscordOperation::AddReaction { message, emoji } => {
                    add_reaction_operation(http, message, emoji).await
                },
                _ => Err(crate::types::AppError::from(DiscordOperationError::MessageSendFailed("未対応の操作".to_string()))),
            }
        })
    };

    // Call the new BattleRecruitmentFacade method with closure pattern
    match facade.new_recruitment(&quest, battle_type, channel_id, guild_id, discord_operation).await {
        Ok(message_id) => {
            ctx.say(format!("募集が正常に作成されました。メッセージID: {}", message_id)).await?;
            Ok(())
        }
        Err(e) => {
            ctx.say(format!("募集作成に失敗しました: {}", e)).await?;
            Err(e)
        }
    }
}

#[poise::command(
    context_menu_command = "recruit_cancel",
    slash_command,
    name_localized("ja", "募集キャンセル"),
    description_localized("ja", "マルチバトル募集をキャンセル")
)]
pub async fn cannel(
    ctx: PoiseContext<'_>,

    #[description = "recruit message"]
    #[description_localized("ja", "募集中のメッセージIDまたはメッセージURL")]
    message: Message,
) -> Result<()> {
    ctx.defer().await?;

    let guild_id = ctx.guild_id().unwrap_or_default().get();
    let channel_id = message.channel_id.get();
    let message_id = message.id.get();

    // Create BattleRecruitmentFacade using AppState (Rustらしいパターン)
    let app_state = &ctx.data().app_state;
    let facade = BattleRecruitmentFacade::new(app_state);

    // Discord操作用のクロージャを作成
    let discord_http = ctx.serenity_context().http.clone();
    let mut discord_operation = |operation: DiscordOperation| -> Pin<Box<dyn Future<Output=Result<DiscordOperationResult>> + Send>> {
        let http = discord_http.clone();
        Box::pin(async move {
            match operation {
                DiscordOperation::DeleteMessage { channel_id, message_id } => {
                    delete_message_operation(http, channel_id, message_id).await
                },
                _ => Err(crate::types::AppError::from(DiscordOperationError::MessageSendFailed("未対応の操作".to_string()))),
            }
        })
    };

    match facade
        .cancel_recruitment(guild_id, channel_id, message_id, discord_operation)
        .await
    {
        Ok(_) => {
            ctx.say("募集が正常にキャンセルされました。").await?;
            Ok(())
        }
        Err(e) => {
            ctx.say(format!("募集キャンセルに失敗しました: {}", e))
                .await?;
            Err(e)
        }
    }
}

async fn quest_auto_complete<'a>(
    _ctx: PoiseContext<'_>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    // Use a static list to avoid borrowing local variables
    const QUEST_LIST: &[&str] = &["Amanda", "Bob", "Christian", "Danny", "Ester", "Falk"];

    // Pre-filter the list synchronously to avoid lifetime issues
    let filtered_items: Vec<String> = QUEST_LIST
        .iter()
        .filter(|name| name.starts_with(partial))
        .map(|name| name.to_string())
        .collect();

    futures::stream::iter(filtered_items)
}

/// SendMessage操作を実行する関数
async fn send_message_operation(
    http: Arc<Http>,
    channel_id: u64,
    content: String,
    embed: Option<CreateEmbed>,
) -> Result<DiscordOperationResult> {
    let channel = ChannelId::from(channel_id);
    let mut builder = CreateMessage::new().content(content);

    if let Some(embed) = embed {
        builder = builder.embed(embed);
    }

    match channel.send_message(&http, builder).await {
        Ok(message) => Ok(DiscordOperationResult {
            message_id: message.id.get(),
            message: Some(message),
        }),
        Err(e) => Err(crate::types::AppError::from(DiscordOperationError::from(e))),
    }
}

/// AddReaction操作を実行する関数
async fn add_reaction_operation(
    http: Arc<Http>,
    message: Message,
    emoji: ReactionType,
) -> Result<DiscordOperationResult> {
    let message_id = message.id.get();
    match message.react(&http, emoji).await {
        Ok(_) => Ok(DiscordOperationResult {
            message_id,
            message: Some(message),
        }),
        Err(e) => Err(crate::types::AppError::from(DiscordOperationError::from(e))),
    }
}

/// DeleteMessage操作を実行する関数
async fn delete_message_operation(
    http: Arc<Http>,
    channel_id: u64,
    message_id: u64,
) -> Result<DiscordOperationResult> {
    let channel = ChannelId::from(channel_id);
    match channel.delete_message(&http, message_id).await {
        Ok(_) => Ok(DiscordOperationResult {
            message_id,
            message: None,
        }),
        Err(e) => Err(crate::types::AppError::from(DiscordOperationError::from(e))),
    }
}
