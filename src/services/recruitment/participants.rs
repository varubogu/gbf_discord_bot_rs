use poise::serenity_prelude::all::{
    ChannelId, Context, CreateEmbed, CreateMessage, EditMessage, MessageId, ReactionType,
};
use sea_orm::DatabaseTransaction;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info};

use crate::models::battle_recruitments::BattleRecruitments;
use crate::repository::battle_recruitments_repository::BattleRecruitmentsRepository;
use crate::repository::database::battle_recruitments_repository::BattleRecruitmentsRepositoryImpl;
use crate::types::{AppError, Result};

/// ParticipantsService - 募集参加者管理を行うサービス
pub struct ParticipantsService {
    battle_recruitment_repo: Arc<BattleRecruitmentsRepositoryImpl>,
}

impl ParticipantsService {
    /// 新しいParticipantsServiceを作成（依存性注入）
    pub fn new(battle_recruitment_repo: Arc<BattleRecruitmentsRepositoryImpl>) -> Self {
        Self {
            battle_recruitment_repo,
        }
    }

    /// 参加者をメッセージIDから更新する（Facade層用メソッド）
    pub async fn update_participants_by_message(
        &self,
        guild_id: u64,
        channel_id: u64,
        message_id: u64,
        txn: &DatabaseTransaction,
    ) -> Result<BattleRecruitments> {
        info!("ParticipantsService::update_participants_by_message - 参加者更新開始");

        // 募集情報の存在確認（トランザクション対応版を使用）
        let recruitment = self
            .battle_recruitment_repo
            .get_by_message_with_txn(txn, guild_id, channel_id, message_id)
            .await?
            .ok_or_else(|| AppError::NotFound("募集が見つかりませんでした".to_string()))?;

        // キャンセル済みの募集は処理を終了
        if recruitment.is_canceled {
            info!(
                recruitment_id = recruitment.id,
                "キャンセル済み募集のため処理をスキップします"
            );
            return Err(AppError::Business {
                message: "この募集はキャンセル済みです".to_string(),
            });
        }

        // 期限切れの募集は処理を終了
        let now = chrono::Utc::now();
        if recruitment.quest_start_at < now {
            info!(
                recruitment_id = recruitment.id,
                quest_start_at = %recruitment.quest_start_at,
                "期限切れ募集のため処理をスキップします"
            );
            return Err(AppError::Business {
                message: "この募集は期限切れです".to_string(),
            });
        }

        info!(recruitment_id = recruitment.id, "参加者更新処理完了");
        Ok(recruitment)
    }

    /// 募集メッセージのリアクションとメンバーを取得
    /// 参加者がいない場合でもリアクション情報を含める
    pub async fn get_reactions_and_members(
        &self,
        ctx: &Context,
        channel_id: u64,
        message_id: u64,
    ) -> Result<HashMap<String, Vec<String>>> {
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

    /// 一意の参加者数を取得（重複排除）
    /// 一人が複数のリアクションをしている場合は1人としてカウント
    pub fn count_unique_participants(
        &self,
        participants_by_reaction: &HashMap<String, Vec<String>>,
    ) -> usize {
        use std::collections::HashSet;

        let mut unique_participants = HashSet::new();
        for users in participants_by_reaction.values() {
            for user_mention in users {
                unique_participants.insert(user_mention.clone());
            }
        }

        unique_participants.len()
    }

    /// すべての参加者のメンションを取得（重複排除）
    pub fn get_all_participants(
        &self,
        participants_by_reaction: &HashMap<String, Vec<String>>,
    ) -> Vec<String> {
        use std::collections::HashSet;

        let mut unique_participants = HashSet::new();
        for users in participants_by_reaction.values() {
            for user_mention in users {
                unique_participants.insert(user_mention.clone());
            }
        }

        unique_participants.into_iter().collect()
    }

    /// メッセージを更新
    pub async fn update_message(
        &self,
        ctx: &Context,
        channel_id: u64,
        message_id: u64,
        content: &str,
        participants_by_reaction: &HashMap<String, Vec<String>>,
    ) -> Result<()> {
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
        let embed = CreateEmbed::new()
            .title("参加者一覧")
            .description(&participants_text)
            .color(0x0099ff);

        // メッセージを更新
        let edit_message = EditMessage::new().content(content).embed(embed);

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

    /// 既に規定人数到達通知が送信されているかチェック
    /// チャンネルの最近のメッセージを確認し、募集メッセージへの返信で
    /// 「参加人数が集まりました」という内容があるかをチェック
    pub async fn has_notification_been_sent(
        &self,
        ctx: &Context,
        channel_id: u64,
        message_id: u64,
    ) -> Result<bool> {
        let channel = ChannelId::from(channel_id);
        let target_message_id = MessageId::from(message_id);

        // チャンネルの最近のメッセージを取得（最大100件）
        match channel
            .messages(
                &ctx.http,
                poise::serenity_prelude::GetMessages::new().limit(100),
            )
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

    /// 規定人数到達時の通知メッセージを送信
    /// 募集メッセージに返信する形で全参加者にメンションを送る
    pub async fn send_recruitment_full_notification(
        &self,
        ctx: &Context,
        channel_id: u64,
        message_id: u64,
        participants: Vec<String>,
    ) -> Result<()> {
        info!(
            "規定人数到達通知を送信: channel_id={}, message_id={}, participants={}",
            channel_id,
            message_id,
            participants.len()
        );

        let channel = ChannelId::from(channel_id);
        let notification_message = format!("{}\n参加人数が集まりました。", participants.join(" "));

        // メッセージIDから参照を作成
        use poise::serenity_prelude::all::MessageReference;
        let reference = MessageReference::from((channel, MessageId::from(message_id)));

        let message = CreateMessage::new()
            .content(notification_message)
            .reference_message(reference);

        match channel.send_message(&ctx.http, message).await {
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
}
