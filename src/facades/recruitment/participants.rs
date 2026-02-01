//! 募集参加者管理Facade層
//!
//! Discord APIとの直接的なやり取りを行い、サービス層のビジネスロジックを呼び出す。

use crate::events::converters::to_edit_message;
use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::repository::database::battle_recruitments_repository::SeaOrmBattleRecruitmentsRepository;
use crate::repository::database::recruitment_participants_repository::SeaOrmRecruitmentParticipantsRepository;
use crate::services::recruitment::participants::ParticipantsService;
use crate::services::recruitment::quest_query_service::QuestQueryService;
use crate::services::recruitment::recruitment_participants_service::RecruitmentParticipantsService;
use crate::services::recruitment::recruitment_query_service::RecruitmentQueryService;
use crate::types;
use crate::types::discord::{EmbedContent, MessageContent};
use crate::utils::discord_helper::send_message_with_optional_reply;
use poise::serenity_prelude::{ChannelId, Context, GetMessages, MessageId, ReactionType};
use sea_orm::TransactionTrait;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info, instrument, warn};

/// 参加者を更新する
///
/// # 引数
/// * `user_id` - リアクション追加/削除を行ったユーザーID（DB登録用、Noneの場合はDB登録なし）
/// * `reaction_emoji` - 追加/削除されたリアクション絵文字（DB登録用）
#[instrument(level = "debug", skip(ctx))]
pub async fn update_participants(
    ctx: &Context,
    guild_id: u64,
    channel_id: u64,
    message_id: u64,
    user_id: Option<u64>,
    reaction_emoji: Option<String>,
    db: &sea_orm::DatabaseConnection,
) -> types::Result<()> {
    info!("BattleRecruitmentFacade::update_participants - 参加者を更新します");
    let txn = db.begin().await?;

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id as i64).await?;

    let result = async {
        // Service層のインスタンスを作成
        let battle_recruitment_repo = Arc::new(SeaOrmBattleRecruitmentsRepository::new());
        let participants_service = ParticipantsService::new(battle_recruitment_repo);
        let query_service = RecruitmentQueryService::new();
        let quest_query_service = QuestQueryService::new();

        // メッセージを取得してv2かどうかを判定
        let channel = poise::serenity_prelude::ChannelId::from(channel_id);
        let message = channel
            .message(&ctx.http, poise::serenity_prelude::MessageId::from(message_id))
            .await?;

        // メッセージにコンポーネント（ボタン）があればv2と判定し、リアクション処理をスキップ
        // v2はボタンで参加管理を行うため、リアクションによる参加者収集は不要
        if !message.components.is_empty() {
            info!("v2募集のためリアクション処理をスキップします: message_id={}", message_id);
            return Ok(());
        }

        // 募集情報の存在確認（キャンセル済み・期限切れチェック含む）
        let recruitment = match participants_service
            .update_participants_by_message(guild_id, channel_id, message_id, &txn)
            .await
        {
            Ok(recruitment) => recruitment,
            Err(types::AppError::Business { message }) => {
                // キャンセル済み・期限切れの場合は静かに処理を終了
                info!("募集更新スキップ: {}", message);
                return Ok(());
            }
            Err(e) => return Err(e),
        };

        // リアクション情報が提供されている場合、DBに登録/削除
        if let (Some(uid), Some(emoji)) = (user_id, reaction_emoji) {
            // battle_styleからリアクション絵文字リストを取得してelement_idを特定
            let battle_style = query_service
                .get_battle_style_by_id(&txn, recruitment.battle_style_id)
                .await?
                .ok_or_else(|| types::AppError::Business {
                    message: "攻略方法が見つかりませんでした".to_string(),
                })?;

            let element_id = if let Some(reactions_str) = &battle_style.reactions {
                let emojis: Vec<&str> = reactions_str.split(',').map(|s| s.trim()).collect();
                emojis
                    .iter()
                    .position(|&e| e == emoji)
                    .map(|pos| (pos + 1) as i32)
            } else {
                None
            };

            // DBに参加者を登録/削除（toggle）
            let participants_repo = SeaOrmRecruitmentParticipantsRepository::new();
            let participants_svc =
                RecruitmentParticipantsService::<SeaOrmRecruitmentParticipantsRepository>::new(
                    Arc::new(participants_repo),
                );

            match participants_svc
                .toggle_participation(&txn, recruitment.id, uid, element_id)
                .await
            {
                Ok(_) => info!(
                    "リアクション参加をDBに登録しました: recruitment_id={}, user_id={}, element_id={:?}",
                    recruitment.id, uid, element_id
                ),
                Err(e) => {
                    warn!("DB登録エラー（続行）: {}", e);
                    // エラーでも続行（embed更新は実行）
                }
            }
        }

        // メッセージのリアクションとユーザーを取得（facade層で直接Discord操作）
        let participants_by_reaction =
            get_reactions_and_members(ctx, channel_id, message_id).await?;

        // 既存のメッセージ内容を取得
        let channel = ChannelId::from(channel_id);
        let message = channel.message(&ctx.http, MessageId::from(message_id)).await?;
        let message_content = message.content.clone();

        // 募集メッセージを編集して参加者一覧部分を反映（facade層で直接Discord操作）
        update_message(
            ctx,
            channel_id,
            message_id,
            &message_content,
            &participants_by_reaction,
        )
        .await?;

        // 規定人数チェック
        let unique_participant_count =
            participants_service.count_unique_participants(&participants_by_reaction);
        info!("現在の参加者数: {}", unique_participant_count);

        // questの規定人数を取得
        if let Ok(quest) = quest_query_service
            .get_quest_by_id(db, recruitment.quest_id)
            .await
        {
            let recruit_count = quest.recruit_count as usize;
            info!("規定人数: {}", recruit_count);

            // 規定人数に達した場合、通知を送信
            if unique_participant_count >= recruit_count {
                // 既に通知が送信されているかチェック（facade層で直接Discord操作）
                let notification_sent =
                    has_notification_been_sent(ctx, channel_id, message_id).await?;

                if !notification_sent {
                    info!("規定人数に到達しました。通知を送信します。");
                    let all_participants =
                        participants_service.get_all_participants(&participants_by_reaction);

                    // 規定人数到達通知送信（facade層で直接Discord操作）
                    send_recruitment_full_notification(
                        ctx,
                        channel_id,
                        message_id,
                        all_participants,
                    )
                    .await?;
                } else {
                    info!("規定人数に達していますが、既に通知済みです。");
                }
            } else {
                info!(
                    "まだ規定人数に達していません。({}/{})",
                    unique_participant_count, recruit_count
                );
            }
        } else {
            warn!(
                "クエスト情報が見つかりませんでした: quest_id={}",
                recruitment.quest_id
            );
        }

        Ok::<(), crate::types::AppError>(())
    }
    .await;

    match result {
        Ok(_) => {
            txn.commit().await?;
            info!(message_id = %message_id, "参加者更新が完了しました");
            Ok(())
        }
        Err(e) => {
            txn.rollback().await?;
            error!(error = %e, message_id = %message_id, "参加者更新エラー");
            Err(e)
        }
    }
}

// ============================================================
// 以下はservice層から移動したDiscord操作関数（facade層で実装）
// ============================================================

/// 募集メッセージのリアクションとメンバーを取得（facade層実装）
///
/// 参加者がいない場合でもリアクション情報を含める
async fn get_reactions_and_members(
    ctx: &Context,
    channel_id: u64,
    message_id: u64,
) -> types::Result<HashMap<String, Vec<String>>> {
    info!(
        "リアクション・メンバー取得開始: channel_id={}, message_id={}",
        channel_id, message_id
    );

    let channel = ChannelId::from(channel_id);
    let message = match channel
        .message(&ctx.http, MessageId::from(message_id))
        .await
    {
        Ok(message) => message,
        Err(e) => {
            error!("メッセージ取得エラー: {:?}", e);
            return Err(format!("Failed to get message: {e}").into());
        }
    };

    let mut participants_by_reaction = HashMap::new();

    for reaction in &message.reactions {
        let reaction_emoji = match &reaction.reaction_type {
            ReactionType::Unicode(emoji) => emoji.clone(),
            ReactionType::Custom { name, .. } => name.clone().unwrap_or_default(),
            _ => continue,
        };

        // リアクションしたユーザーを取得
        match message
            .reaction_users(&ctx.http, reaction.reaction_type.clone(), Some(100), None)
            .await
        {
            Ok(users) => {
                let user_mentions: Vec<String> = users
                    .iter()
                    .filter(|user| !user.bot) // ボットユーザーを除外
                    .map(|user| format!("<@{}>", user.id))
                    .collect();

                // 参加者がいない場合でも空のVecとして追加
                participants_by_reaction.insert(reaction_emoji, user_mentions);
            }
            Err(e) => {
                error!("リアクションユーザー取得エラー: {:?}", e);
                // エラーが発生しても他のリアクションの処理は続行
            }
        }
    }

    info!(
        "リアクション参加者取得完了: {} reactions found",
        participants_by_reaction.len()
    );
    Ok(participants_by_reaction)
}

/// メッセージを更新（facade層実装）
async fn update_message(
    ctx: &Context,
    channel_id: u64,
    message_id: u64,
    content: &str,
    participants_by_reaction: &HashMap<String, Vec<String>>,
) -> types::Result<()> {
    info!(
        "メッセージ更新開始: channel_id={}, message_id={}",
        channel_id, message_id
    );

    let channel = ChannelId::from(channel_id);
    let mut message = match channel
        .message(&ctx.http, MessageId::from(message_id))
        .await
    {
        Ok(message) => message,
        Err(e) => {
            error!("更新対象メッセージ取得エラー: {:?}", e);
            return Err(format!("Failed to get message for update: {e}").into());
        }
    };

    // 参加者情報を埋め込みに変換
    let participants_text = if participants_by_reaction.is_empty() {
        "現在参加者はいません。".to_string()
    } else {
        // BattleType絵文字の順序で表示（🔥💧🌱🌪️✨🌑）
        let emoji_order = vec!["🔥", "💧", "🌱", "🌪️", "✨", "🌑"];
        let mut text = String::new();

        // emoji_orderに含まれる絵文字を順序通りに表示
        for emoji in &emoji_order {
            if let Some(users) = participants_by_reaction.get(*emoji) {
                if users.is_empty() {
                    text.push_str(&format!("{emoji} なし\n"));
                } else {
                    text.push_str(&format!("{} {}\n", emoji, users.join(" ")));
                }
            }
        }

        // emoji_orderに含まれない絵文字も追加（カスタム絵文字など）
        for (emoji, users) in participants_by_reaction {
            if !emoji_order.contains(&emoji.as_str()) {
                if users.is_empty() {
                    text.push_str(&format!("{emoji} なし\n"));
                } else {
                    text.push_str(&format!("{} {}\n", emoji, users.join(" ")));
                }
            }
        }

        if text.is_empty() {
            "現在参加者はいません。".to_string()
        } else {
            text
        }
    };

    // 埋め込みメッセージを作成
    let embed_content = EmbedContent::new()
        .with_title("参加者一覧")
        .with_description(&participants_text)
        .with_color(0x0099ff);

    // メッセージを更新
    let message_content = MessageContent::new()
        .with_text(content)
        .with_embed(embed_content);
    let edit_message = to_edit_message(&message_content);

    match message.edit(&ctx.http, edit_message).await {
        Ok(_) => {
            info!("メッセージ更新成功: message_id={}", message_id);
            Ok(())
        }
        Err(e) => {
            error!("メッセージ更新エラー: {:?}", e);
            Err(format!("Failed to update message: {e}").into())
        }
    }
}

/// 既に規定人数到達通知が送信されているかチェック（facade層実装）
///
/// チャンネルの最近のメッセージを確認し、募集メッセージへの返信で
/// 「参加人数が集まりました」という内容があるかをチェック
async fn has_notification_been_sent(
    ctx: &Context,
    channel_id: u64,
    message_id: u64,
) -> types::Result<bool> {
    let channel = ChannelId::from(channel_id);
    let target_message_id = MessageId::from(message_id);

    // チャンネルの最近のメッセージを取得（最大100件）
    match channel
        .messages(&ctx.http, GetMessages::new().limit(100))
        .await
    {
        Ok(messages) => {
            // 募集メッセージへの返信で「参加人数が集まりました」を含むメッセージを探す
            for msg in messages {
                if let Some(ref_msg) = &msg.referenced_message {
                    if ref_msg.id == target_message_id
                        && msg.content.contains("参加人数が集まりました")
                    {
                        info!("既に規定人数到達通知が送信済みです");
                        return Ok(true);
                    }
                }
            }
            Ok(false)
        }
        Err(e) => {
            error!("メッセージ履歴取得エラー: {:?}", e);
            // エラーの場合は安全側に倒して、通知済みとして扱わない
            Ok(false)
        }
    }
}

/// 規定人数到達時の通知メッセージを送信（facade層実装）
///
/// 募集メッセージに返信する形で全参加者にメンションを送る
async fn send_recruitment_full_notification(
    ctx: &Context,
    channel_id: u64,
    message_id: u64,
    participants: Vec<String>,
) -> types::Result<()> {
    info!(
        "規定人数到達通知を送信: channel_id={}, message_id={}, participants={}",
        channel_id,
        message_id,
        participants.len()
    );

    let channel = ChannelId::from(channel_id);
    let notification_message = format!("{}\n参加人数が集まりました。", participants.join(" "));

    // 返信形式で送信を試み、失敗時は文脈情報を付加して通常メッセージとして送信
    match send_message_with_optional_reply(
        &ctx.http,
        channel,
        MessageId::from(message_id),
        notification_message,
        Some("規定人数到達通知".to_string()),
    )
    .await
    {
        Ok(_) => {
            info!("規定人数到達通知送信成功");
            Ok(())
        }
        Err(e) => {
            error!("規定人数到達通知送信エラー: {:?}", e);
            Err(format!("Failed to send notification: {e}").into())
        }
    }
}
