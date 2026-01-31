use poise::serenity_prelude::all::{
    ChannelId, ComponentInteraction, EditInteractionResponse, Message, MessageId,
};
use sea_orm::{ConnectionTrait, DatabaseTransaction};
use std::collections::HashMap;
use tracing::{error, info, warn};

use crate::models::battle_recruitments::BattleRecruitments;
use crate::services::message::MessageService;
use crate::services::message::MessageTextId;
use crate::types::discord::{DiscordChannelId, DiscordGuildId, DiscordMessageId};
use crate::types::domain_interface_result::CanCancelResult;
use crate::types::{AppError, PoiseContext, Result};

/// キャンセル可能性をチェックし、結果を返す
pub async fn check_can_cancel_recruitment<R: crate::repository::BattleRecruitmentsRepository>(
    ctx: PoiseContext<'_>,
    message: &Message,
    battle_recruitment_repo: &R,
    txn: &DatabaseTransaction,
) -> Result<CanCancelResult> {
    let guild_id = if let Some(guild_id) = ctx.guild_id() {
        guild_id.get()
    } else {
        warn!("guild_idを取得できませんでした");
        return Err(AppError::Business {
            message: "ギルド情報を取得できませんでした".to_string(),
        });
    };
    let channel_id = message.channel_id;
    let message_id = message.id;

    info!(
        "キャンセル可能性チェック開始: guild_id={}, channel_id={}, message_id={}",
        guild_id, channel_id, message_id
    );

    // DBから募集情報を取得（エラーの場合はNone扱い）（トランザクション対応版を使用）
    // serenity型からドメイン型に変換
    let recruitment_opt = battle_recruitment_repo
        .get_by_message_with_txn(
            txn,
            DiscordGuildId::new(guild_id),
            DiscordChannelId::new(channel_id.get()),
            DiscordMessageId::new(message_id.get()),
        )
        .await?;

    // Discordからメッセージを取得（エラーの場合はNone扱い）
    let discord_message_opt = get_discord_message(ctx, channel_id.into(), message_id.into()).await;

    let result = match (recruitment_opt, discord_message_opt) {
        // DBあり + メッセージあり
        (Some(recruitment), Ok(_)) => {
            if recruitment.is_canceled {
                CanCancelResult::AlreadyCancelled
            } else {
                // 開催日時を過ぎているかチェック
                let now = chrono::Utc::now();
                if recruitment.quest_start_at <= now {
                    CanCancelResult::EventDatePassed
                } else {
                    CanCancelResult::Success
                }
            }
        }
        // DBあり + メッセージなし
        (Some(_), Err(_)) => CanCancelResult::MessageDeleted,
        // DBなし + メッセージあり
        (None, Ok(_)) => CanCancelResult::NotRecruitMessage,
        // DBなし + メッセージなし
        (None, Err(_)) => CanCancelResult::NotFound,
    };

    info!("キャンセル可能性チェック完了: {:?}", result);
    Ok(result)
}

/// Discordからメッセージを取得
pub async fn get_discord_message(
    ctx: PoiseContext<'_>,
    channel_id: u64,
    message_id: u64,
) -> Result<Message> {
    let channel = ChannelId::from(channel_id);
    let message = channel
        .message(&ctx.http(), MessageId::from(message_id))
        .await?;
    Ok(message)
}

/// メッセージIDから募集をキャンセルする
pub async fn cancel_recruitment_by_message<R: crate::repository::BattleRecruitmentsRepository>(
    txn: &DatabaseTransaction,
    battle_recruitment_repo: &R,
    guild_id: u64,
    channel_id: u64,
    message_id: u64,
    cancel_message_id: DiscordMessageId,
) -> Result<BattleRecruitments> {
    info!("cancel_recruitment_by_message - キャンセル処理開始");

    // 募集情報の存在確認（トランザクション対応版を使用）
    let recruitment = get_recruitment_from_database(
        guild_id,
        channel_id,
        message_id,
        battle_recruitment_repo,
        txn,
    )
    .await?;

    // 募集をキャンセル済み状態に更新
    mark_recruitment_as_cancelled(
        txn,
        recruitment.id,
        cancel_message_id,
        battle_recruitment_repo,
    )
    .await?;

    info!(recruitment_id = recruitment.id, "キャンセル処理完了");
    Ok(recruitment)
}

/// DBから募集情報を取得
pub async fn get_recruitment_from_database<R: crate::repository::BattleRecruitmentsRepository>(
    guild_id: u64,
    channel_id: u64,
    message_id: u64,
    battle_recruitment_repo: &R,
    txn: &DatabaseTransaction,
) -> Result<BattleRecruitments> {
    info!(
        "DB募集情報取得開始: guild_id={}, channel_id={}, message_id={}",
        guild_id, channel_id, message_id
    );

    // u64をドメイン型に変換
    match battle_recruitment_repo
        .get_by_message_with_txn(
            txn,
            DiscordGuildId::new(guild_id),
            DiscordChannelId::new(channel_id),
            DiscordMessageId::new(message_id),
        )
        .await?
    {
        Some(recruitment) => {
            info!("募集情報取得成功: recruitment_id={}", recruitment.id);
            Ok(recruitment)
        }
        None => {
            warn!("募集情報が見つかりません: message_id={}", message_id);
            Err(AppError::Business {
                message: format!("Recruitment not found for message_id: {message_id}"),
            })
        }
    }
}

/// 募集をキャンセル済み状態に更新
pub async fn mark_recruitment_as_cancelled<R: crate::repository::BattleRecruitmentsRepository>(
    txn: &DatabaseTransaction,
    recruitment_id: i32,
    cancel_message_id: DiscordMessageId,
    battle_recruitment_repo: &R,
) -> Result<()> {
    info!(
        "募集キャンセル済み状態更新: recruitment_id={}",
        recruitment_id
    );

    // 終了メッセージID = 0 でキャンセル状態を表現
    battle_recruitment_repo
        .set_canceled_with_txn(txn, recruitment_id, cancel_message_id)
        .await?;

    info!(
        "募集キャンセル済み状態更新完了: recruitment_id={}",
        recruitment_id
    );
    Ok(())
}

/// リアクションから参加者一覧取得
pub async fn get_participants_from_reactions(
    ctx: PoiseContext<'_>,
    channel_id: u64,
    message_id: u64,
) -> Result<Vec<String>> {
    info!(
        "リアクション参加者取得開始: channel_id={}, message_id={}",
        channel_id, message_id
    );

    let channel = ChannelId::from(channel_id);
    let message = channel
        .message(&ctx.http(), MessageId::from(message_id))
        .await?;

    let mut all_participants = Vec::new();

    for reaction in &message.reactions {
        // リアクションしたユーザーを取得
        match message
            .reaction_users(&ctx.http(), reaction.reaction_type.clone(), Some(100), None)
            .await
        {
            Ok(users) => {
                let user_mentions: Vec<String> = users
                    .iter()
                    .filter(|user| !user.bot) // ボットユーザーを除外
                    .map(|user| format!("<@{}>", user.id))
                    .collect();

                all_participants.extend(user_mentions);
            }
            Err(e) => {
                error!("リアクションユーザー取得エラー: {:?}", e);
                // エラーが発生しても他のリアクションの処理は続行
            }
        }
    }

    // 重複を除去
    all_participants.sort();
    all_participants.dedup();

    info!(
        "リアクション参加者取得完了: {} participants found",
        all_participants.len()
    );
    Ok(all_participants)
}

/// キャンセル済みメッセージ作成
pub async fn create_cancelled_message_content<C>(
    db: &C,
    message_service: &MessageService,
    guild_id: Option<i64>,
    locale: Option<&str>,
    original_content: &str,
) -> Result<String>
where
    C: ConnectionTrait,
{
    // メッセージサービスからキャンセル済みサフィックスを取得
    let cancelled_suffix = message_service
        .get_message(
            db,
            MessageTextId::RecruitmentCommandCancelledMessageSuffix.as_str(),
            HashMap::new(),
            guild_id,
            locale,
        )
        .await
        .unwrap_or_else(|_| "この募集はキャンセルされました".to_string());

    // 元のメッセージに打ち消し線と「キャンセル済み」を追加
    Ok(format!("~~{original_content}~~\n\n**{cancelled_suffix}**"))
}

/// キャンセル通知メッセージ作成
pub async fn create_cancel_notification_text<C>(
    db: &C,
    message_service: &MessageService,
    guild_id: Option<i64>,
    locale: Option<&str>,
    participants: &[String],
) -> Result<String>
where
    C: ConnectionTrait,
{
    if participants.is_empty() {
        // 参加者なしの場合
        let message = message_service
            .get_message(
                db,
                MessageTextId::RecruitmentCommandCancelNotificationNoParticipants.as_str(),
                HashMap::new(),
                guild_id,
                locale,
            )
            .await
            .unwrap_or_else(|_| "募集がキャンセルされました。".to_string());
        Ok(message)
    } else {
        // 参加者ありの場合
        let base_message = message_service
            .get_message(
                db,
                MessageTextId::RecruitmentCommandCancelNotificationWithParticipants.as_str(),
                HashMap::new(),
                guild_id,
                locale,
            )
            .await
            .unwrap_or_else(|_| "募集がキャンセルされました。\n参加予定だった皆さん".to_string());

        let participants_str = participants.join(" ");
        Ok(format!("{base_message}: {participants_str}"))
    }
}

/// キャンセル返信送信
pub async fn send_cancel_reply_message(
    ctx: PoiseContext<'_>,
    channel_id: u64,
    original_message_id: u64,
    content: &str,
) -> Result<MessageId> {
    info!(
        "キャンセル返信送信: channel_id={}, original_message_id={}",
        channel_id, original_message_id
    );

    let cancel_reply = ctx.say(content).await?;
    info!("キャンセル返信送信完了");

    Ok(cancel_reply.message().await?.id)
}

/// 確認メッセージを削除する
pub async fn delete_confirmation_message(
    ctx: PoiseContext<'_>,
    interaction: &ComponentInteraction,
) -> Result<()> {
    info!("確認メッセージを削除します");

    // メッセージを削除
    interaction
        .message
        .delete(&ctx.serenity_context().http)
        .await?;

    info!("確認メッセージの削除完了");
    Ok(())
}

/// キャンセル中表示に変更する
pub async fn show_cancelling_message<C>(
    ctx: PoiseContext<'_>,
    interaction: &ComponentInteraction,
    db: &C,
    message_service: &MessageService,
    guild_id: Option<i64>,
    locale: Option<&str>,
) -> Result<()>
where
    C: ConnectionTrait,
{
    info!("キャンセル中表示に変更します");

    // メッセージサービスからキャンセル中メッセージを取得
    let cancelling_message = message_service
        .get_message(
            db,
            MessageTextId::RecruitmentCommandCancellingProgress.as_str(),
            HashMap::new(),
            guild_id,
            locale,
        )
        .await
        .unwrap_or_else(|_| "キャンセル中...".to_string());

    interaction
        .edit_response(
            &ctx.serenity_context().http,
            EditInteractionResponse::new()
                .content(cancelling_message)
                .components(vec![]),
        )
        .await?;

    info!("キャンセル中表示への変更完了");
    Ok(())
}

/// キャンセル中メッセージを削除する
pub async fn delete_cancelling_message(
    ctx: PoiseContext<'_>,
    interaction: &ComponentInteraction,
) -> Result<()> {
    info!("キャンセル中メッセージを削除します");

    interaction
        .message
        .delete(&ctx.serenity_context().http)
        .await?;

    info!("キャンセル中メッセージの削除完了");
    Ok(())
}
