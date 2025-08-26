use poise::serenity_prelude::all::{
    ChannelId, Context, CreateEmbed, EditMessage, Message, MessageId, ReactionType,
};
use sea_orm::DatabaseTransaction;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::models::battle_recruitments::BattleRecruitments;
use crate::repository::BattleRecruitmentsRepository;
use crate::types::battle_type::BattleType;
use crate::types::{AppError, Result};

pub(crate) struct PaticipantsParameter {
    pub guild_id: i64,
    pub channel_id: i64,
}

/// ParticipantsService - 募集参加者管理を行うサービス
pub struct ParticipantsService {
    battle_recruitment_repo: Arc<dyn BattleRecruitmentsRepository>,
}

impl ParticipantsService {
    /// 新しいParticipantsServiceを作成（依存性注入）
    pub fn new(battle_recruitment_repo: Arc<dyn BattleRecruitmentsRepository>) -> Self {
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

        // 募集情報の存在確認
        let recruitment = self
            .battle_recruitment_repo
            .get_by_message(guild_id as i64, channel_id as i64, message_id as i64)
            .await?
            .ok_or_else(|| AppError::NotFound("募集が見つかりませんでした".to_string()))?;

        info!(recruitment_id = recruitment.id, "参加者更新処理完了");
        Ok(recruitment)
    }

    /// 募集メッセージのリアクションとメンバーを取得
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
                return Err(format!("Failed to get message: {}", e).into());
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

                    if !user_mentions.is_empty() {
                        participants_by_reaction.insert(reaction_emoji, user_mentions);
                    }
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

    /// DBから募集情報を取得
    pub async fn get_recruitment_from_db(
        &self,
        guild_id: u64,
        channel_id: u64,
        message_id: u64,
    ) -> Result<BattleRecruitments> {
        panic!();
        // info!("DB募集情報取得開始: guild_id={}, channel_id={}, message_id={}",
        //       guild_id, channel_id, message_id);
        //
        // match self.battle_recruitment_repo
        //     .get_by_message(guild_id as i64, channel_id as i64, message_id as i64)
        //     .await
        // {
        //     Ok(Some(recruitment)) => {
        //         info!("募集情報取得成功: recruitment_id={}", recruitment.id);
        //         Ok(recruitment)
        //     }
        //     Ok(None) => {
        //         warn!("募集情報が見つかりません: message_id={}", message_id);
        //         Err(format!("Recruitment not found for message_id: {}", message_id))
        //     }
        //     Err(e) => {
        //         error!("DB募集情報取得エラー: {:?}", e);
        //         Err(format!("Database error: {}", e))
        //     }
        // }
    }

    /// リアクションとメンバーからメッセージを作成
    pub async fn create_participant_message(
        &self,
        participants: &[String],
        quest_name: &str,
    ) -> Result<String> {
        warn!("ParticipantsService::create_participant_message - 仕様検討中です");
        info!("参加者メッセージ作成をエミュレート");

        let participant_list = if participants.is_empty() {
            "現在参加者はいません".to_string()
        } else {
            participants.join("\n")
        };

        let message = format!("{}の参加者一覧\n\n{}", quest_name, participant_list);
        Ok(message)
    }

    /// クエストと日時からメッセージを作成（参加者情報含む）
    pub async fn create_quest_datetime_message(
        &self,
        quest_name: &str,
        datetime: &str,
        participants: &[String],
    ) -> Result<String> {
        warn!("ParticipantsService::create_quest_datetime_message - 仕様検討中です");
        info!("クエスト・日時メッセージ作成をエミュレート");

        let participant_count = participants.len();
        let message = format!(
            "{}の募集\n開催日時: {}\n参加者数: {}名\n\n参加者:\n{}",
            quest_name,
            datetime,
            participant_count,
            if participants.is_empty() {
                "なし".to_string()
            } else {
                participants.join("\n")
            }
        );
        Ok(message)
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
                return Err(format!("Failed to get message for update: {}", e).into());
            }
        };

        // 参加者情報を埋め込みに変換
        let participants_text = if participants_by_reaction.is_empty() {
            "現在参加者はいません。".to_string()
        } else {
            let mut text = String::new();
            for (emoji, users) in participants_by_reaction {
                text.push_str(&format!("{} {}\n", emoji, users.join(" ")));
            }
            text
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
                Err(format!("Failed to update message: {}", e).into())
            }
        }
    }
}
