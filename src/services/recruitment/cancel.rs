use poise::serenity_prelude::all::{
    ChannelId, Context, CreateMessage, EditMessage, Message, MessageId,
};
use sea_orm::DatabaseTransaction;
use tracing::{error, info, warn};

use crate::models::battle_recruitments::BattleRecruitments;
use crate::types::{AppError, PoiseContext, Result};

/// キャンセル可能性チェックの結果
#[derive(Debug, PartialEq)]
pub enum CanCancelResult {
    /// キャンセル可能
    Success,
    /// 既にキャンセル済み
    AlreadyCancelled,
    /// 募集メッセージは過去にあったが、削除済み
    MessageDeleted,
    /// 募集メッセージじゃない
    NotRecruitMessage,
    /// 募集がなく、メッセージもない
    NotFound,
}

/// キャンセル可能性をチェックし、結果を返す
pub async fn check_can_cancel_recruitment(
    ctx: &Context,
    guild_id: u64,
    channel_id: u64,
    message_id: u64,
    battle_recruitment_repo: &dyn crate::repository::BattleRecruitmentsRepository,
) -> Result<CanCancelResult> {
    info!(
        "キャンセル可能性チェック開始: guild_id={}, channel_id={}, message_id={}",
        guild_id, channel_id, message_id
    );

    // DBから募集情報を取得（エラーの場合はNone扱い）
    let recruitment_opt = battle_recruitment_repo
        .get_by_message(guild_id, channel_id, message_id)
        .await?;

    // Discordからメッセージを取得（エラーの場合はNone扱い）
    let discord_message_opt = get_discord_message(ctx, channel_id, message_id).await;

    let result = match (recruitment_opt, discord_message_opt) {
        // DBあり + メッセージあり
        (Some(recruitment), Ok(_)) => {
            if recruitment.is_canceled {
                CanCancelResult::AlreadyCancelled
            } else {
                CanCancelResult::Success
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
    ctx: &Context,
    channel_id: u64,
    message_id: u64,
) -> Result<Message> {
    let channel = ChannelId::from(channel_id);
    let message = channel
        .message(&ctx.http, MessageId::from(message_id))
        .await?;
    Ok(message)
}

/// メッセージIDから募集をキャンセルする
pub async fn cancel_recruitment_by_message(
    txn: &DatabaseTransaction,
    battle_recruitment_repo: &dyn crate::repository::BattleRecruitmentsRepository,
    guild_id: u64,
    channel_id: u64,
    message_id: u64,
    cancel_message_id: MessageId,
) -> Result<BattleRecruitments> {
    info!("cancel_recruitment_by_message - キャンセル処理開始");

    // 募集情報の存在確認
    let recruitment =
        get_recruitment_from_database(guild_id, channel_id, message_id, battle_recruitment_repo)
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
pub async fn get_recruitment_from_database(
    guild_id: u64,
    channel_id: u64,
    message_id: u64,
    battle_recruitment_repo: &dyn crate::repository::BattleRecruitmentsRepository,
) -> Result<BattleRecruitments> {
    info!(
        "DB募集情報取得開始: guild_id={}, channel_id={}, message_id={}",
        guild_id, channel_id, message_id
    );

    match battle_recruitment_repo
        .get_by_message(guild_id, channel_id, message_id)
        .await?
    {
        Some(recruitment) => {
            info!("募集情報取得成功: recruitment_id={}", recruitment.id);
            Ok(recruitment)
        }
        None => {
            warn!("募集情報が見つかりません: message_id={}", message_id);
            Err(AppError::Business {
                message: format!("Recruitment not found for message_id: {}", message_id),
            })
        }
    }
}

/// 募集をキャンセル済み状態に更新
pub async fn mark_recruitment_as_cancelled(
    txn: &DatabaseTransaction,
    recruitment_id: i32,
    cancel_message_id: MessageId,
    battle_recruitment_repo: &dyn crate::repository::BattleRecruitmentsRepository,
) -> Result<()> {
    info!(
        "募集キャンセル済み状態更新: recruitment_id={}",
        recruitment_id
    );

    // 終了メッセージID = 0 でキャンセル状態を表現
    battle_recruitment_repo
        .set_canceled(recruitment_id, cancel_message_id)
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
pub async fn create_cancelled_message_content(original_content: &str) -> Result<String> {
    warn!("create_cancelled_message_content - 仕様検討中です");
    // 暫定実装：元のメッセージに「キャンセル済み」を追加
    Ok(format!(
        "~~{}~~\n\n**この募集はキャンセルされました**",
        original_content
    ))
}

/// キャンセル通知メッセージ作成
pub async fn create_cancel_notification_text(participants: &[String]) -> Result<String> {
    warn!("create_cancel_notification_text - 仕様検討中です");

    if participants.is_empty() {
        Ok("募集がキャンセルされました。".to_string())
    } else {
        let participants_str = participants.join(" ");
        Ok(format!(
            "募集がキャンセルされました。\n参加予定だった皆さん: {}",
            participants_str
        ))
    }
}

/// 元のメッセージをキャンセル済みに編集
pub async fn edit_original_message_as_cancelled(
    ctx: PoiseContext<'_>,
    channel_id: u64,
    message_id: u64,
    original_content: &str,
) -> Result<()> {
    info!(
        "元メッセージをキャンセル済みに編集: channel_id={}, message_id={}",
        channel_id, message_id
    );

    let channel = ChannelId::from(channel_id);
    let cancelled_content = create_cancelled_message_content(original_content).await?;

    let edit_message = EditMessage::new().content(cancelled_content);

    channel
        .edit_message(&ctx.http(), MessageId::from(message_id), edit_message)
        .await?;

    info!("元メッセージのキャンセル済み編集完了");
    Ok(())
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
