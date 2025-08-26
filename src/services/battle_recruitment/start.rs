use poise::serenity_prelude::all::{
    ChannelId, Context, CreateMessage, Message, MessageId, ReactionType,
};
use sea_orm::DatabaseTransaction;
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::models::battle_recruitments::BattleRecruitments;
use crate::repository::BattleRecruitmentsRepository;
use crate::types::{AppError, Result};

/// StartRecruitmentService - 募集開始処理を行うサービス
pub struct StartRecruitmentService {
    repo: Arc<dyn BattleRecruitmentsRepository>,
}

impl StartRecruitmentService {
    /// 新しいStartRecruitmentServiceを作成（依存性注入）
    pub fn new(repo: Arc<dyn BattleRecruitmentsRepository>) -> Self {
        Self { repo }
    }

    /// メッセージIDから募集を開始する（Facade層用メソッド）
    pub async fn start_by_message(
        &self,
        guild_id: u64,
        channel_id: u64,
        message_id: u64,
        txn: &DatabaseTransaction,
    ) -> Result<BattleRecruitments> {
        info!("StartRecruitmentService::start_by_message - 開始処理開始");

        // 募集情報の存在確認
        let recruitment = self
            .get_recruitment_from_db(guild_id, channel_id, message_id)
            .await?;

        // 募集を開始済み状態に更新（message_idを使用）
        self.mark_recruitment_as_started(recruitment.id as i64, message_id)
            .await?;

        info!(recruitment_id = recruitment.id, "開始処理完了");
        Ok(recruitment)
    }

    /// DBから募集情報を取得
    pub async fn get_recruitment_from_db(
        &self,
        guild_id: u64,
        channel_id: u64,
        message_id: u64,
    ) -> Result<BattleRecruitments> {
        info!(
            "DB募集情報取得開始: guild_id={}, channel_id={}, message_id={}",
            guild_id, channel_id, message_id
        );

        match self
            .repo
            .get_by_message(guild_id as i64, channel_id as i64, message_id as i64)
            .await?
        {
            Some(recruitment) => {
                info!("募集情報取得成功: recruitment_id={}", recruitment.id);
                Ok(recruitment)
            }
            None => {
                warn!("募集情報が見つかりません: message_id={}", message_id);
                Err(AppError::NotFound(format!(
                    "Recruitment not found for message_id: {}",
                    message_id
                )))
            }
        }
    }

    /// リアクションから参加者一覧取得
    pub async fn get_participants_from_reactions(
        &self,
        ctx: &Context,
        channel_id: u64,
        message_id: u64,
    ) -> Result<Vec<String>> {
        info!(
            "リアクション参加者取得開始: channel_id={}, message_id={}",
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

        let mut all_participants = Vec::new();

        for reaction in &message.reactions {
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

    /// 開始メッセージを作成（参加者へのメンション含む）
    pub async fn create_start_message(
        &self,
        quest_name: &str,
        participants: &[String],
    ) -> Result<String> {
        warn!("StartRecruitmentService::create_start_message - 仕様検討中です");
        info!("開始メッセージ作成をエミュレート");

        let participant_mentions = if participants.is_empty() {
            "参加者がいません".to_string()
        } else {
            participants.join(" ")
        };

        let message = format!(
            "🚀 **クエスト出発時間です！** 🚀\n\n{}\n\n参加者の皆さん: {}\n\nクエストを開始してください！",
            quest_name, participant_mentions
        );
        Ok(message)
    }

    /// 元の募集メッセージに返信する形でメッセージを送信
    pub async fn send_start_reply(
        &self,
        ctx: &Context,
        channel_id: u64,
        original_message_id: u64,
        content: &str,
    ) -> Result<()> {
        info!(
            "開始返信送信開始: channel_id={}, message_id={}",
            channel_id, original_message_id
        );

        let channel = ChannelId::from(channel_id);
        let original_message = MessageId::from(original_message_id);

        let reply_message = CreateMessage::new()
            .content(content)
            .reference_message((channel, original_message));

        match channel.send_message(&ctx.http, reply_message).await {
            Ok(sent_message) => {
                info!("開始返信送信成功: sent_message_id={}", sent_message.id);
                Ok(())
            }
            Err(e) => {
                error!("開始返信送信エラー: {:?}", e);
                Err(format!("Failed to send start reply: {}", e).into())
            }
        }
    }

    /// 募集を開始済み状態に更新
    /// 注意: 現在のBattleRecruitmentRepositoryトレイトには開始済み状態更新メソッドがないため、
    /// set_end_messageを使用して終了メッセージIDを設定することで開始状態を表現します。
    pub async fn mark_recruitment_as_started(
        &self,
        recruitment_id: i64,
        end_message_id: u64,
    ) -> Result<()> {
        info!(
            "募集開始済み状態更新開始: recruitment_id={}, end_message_id={}",
            recruitment_id, end_message_id
        );

        self.repo
            .set_end_message(recruitment_id as i32, end_message_id as i64)
            .await?;

        info!(
            "募集開始済み状態更新成功: recruitment_id={}",
            recruitment_id
        );
        Ok(())
    }
}
