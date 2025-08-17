use std::sync::Arc;
use poise::serenity_prelude::all::{Context, Message, ChannelId, MessageId, CreateMessage, EditMessage, ReactionType};
use poise::serenity_prelude::GuildId;
use tracing::{info, warn, error};

use crate::repository::BattleRecruitmentRepository;
use crate::infrastructure::database::Transaction;
use crate::models::battle_recruitment::BattleRecruitment;
use crate::types::PoiseError;

pub(crate) struct CancelParameter {
    pub guild_id: i64,
    pub channel_id: i64,
    pub message_id: i64,
}

/// CancelRecruitmentService - 募集キャンセル処理を行うサービス（Repository依存注入対応）
pub struct CancelRecruitmentService<'a> {
    repo: &'a dyn BattleRecruitmentRepository,
}

impl<'a> CancelRecruitmentService<'a> {
    pub fn new(repo: &'a dyn BattleRecruitmentRepository) -> Self {
        Self { repo }
    }

    /// DBから募集情報を取得（トランザクション内）
    pub async fn get_recruitment_from_db_with_txn(
        &self,
        txn: &Transaction,
        guild_id: u64,
        channel_id: u64,
        message_id: u64,
    ) -> Result<BattleRecruitment, PoiseError> {
        info!("DB募集情報取得開始: guild_id={}, channel_id={}, message_id={}", 
              guild_id, channel_id, message_id);
        
        match self.repo.get_by_message_with_txn(
            txn,
            guild_id, 
            channel_id, 
            message_id
        ).await? {
            Some(recruitment) => {
                info!("募集情報取得成功: recruitment_id={}", recruitment.id);
                Ok(recruitment)
            }
            None => {
                warn!("募集情報が見つかりません: message_id={}", message_id);
                Err(PoiseError::from(format!("Recruitment not found for message_id: {}", message_id)))
            }
        }
    }

    /// 募集をキャンセル済み状態に更新（トランザクション内）
    pub async fn mark_recruitment_as_cancelled_with_txn(
        &self,
        txn: &Transaction,
        recruitment_id: i32,
    ) -> Result<(), PoiseError> {
        info!("募集キャンセル済み状態更新: recruitment_id={}", recruitment_id);
        
        // 終了メッセージID = 0 でキャンセル状態を表現
        self.repo.set_end_message_with_txn(txn, recruitment_id, 0).await?;
        
        info!("募集キャンセル済み状態更新完了: recruitment_id={}", recruitment_id);
        Ok(())
    }

    /// リアクションから参加者一覧取得（非トランザクション）
    pub async fn get_participants_from_reactions(
        &self, 
        ctx: &Context, 
        channel_id: u64, 
        message_id: u64
    ) -> Result<Vec<String>, PoiseError> {
        info!("リアクション参加者取得開始: channel_id={}, message_id={}", channel_id, message_id);
        
        let channel = ChannelId::from(channel_id);
        let message = channel.message(&ctx.http, MessageId::from(message_id)).await?;

        let mut all_participants = Vec::new();

        for reaction in &message.reactions {
            // リアクションしたユーザーを取得
            match message.reaction_users(&ctx.http, reaction.reaction_type.clone(), Some(100), None).await {
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

        info!("リアクション参加者取得完了: {} participants found", all_participants.len());
        Ok(all_participants)
    }

    /// キャンセル済みメッセージ作成（非トランザクション）
    pub async fn create_cancelled_message(&self, original_content: &str) -> Result<String, PoiseError> {
        warn!("CancelRecruitmentService::create_cancelled_message - 仕様検討中です");
        // 暫定実装：元のメッセージに「キャンセル済み」を追加
        Ok(format!("~~{}~~\n\n**この募集はキャンセルされました**", original_content))
    }

    /// キャンセル通知メッセージ作成（非トランザクション）
    pub async fn create_cancel_notification(&self, participants: &[String]) -> Result<String, PoiseError> {
        warn!("CancelRecruitmentService::create_cancel_notification - 仕様検討中です");
        
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

    /// 元のメッセージをキャンセル済みに編集（非トランザクション）
    pub async fn edit_original_message_as_cancelled(
        &self, 
        ctx: &Context, 
        channel_id: u64, 
        message_id: u64, 
        original_content: &str
    ) -> Result<(), PoiseError> {
        info!("元メッセージをキャンセル済みに編集: channel_id={}, message_id={}", 
              channel_id, message_id);
        
        let channel = ChannelId::from(channel_id);
        let cancelled_content = self.create_cancelled_message(original_content).await?;
        
        let edit_message = EditMessage::new().content(cancelled_content);
        
        channel.edit_message(&ctx.http, MessageId::from(message_id), edit_message).await?;
        
        info!("元メッセージのキャンセル済み編集完了");
        Ok(())
    }

    /// キャンセル返信送信（非トランザクション）
    pub async fn send_cancel_reply(
        &self, 
        ctx: &Context, 
        channel_id: u64, 
        original_message_id: u64, 
        content: &str
    ) -> Result<(), PoiseError> {
        info!("キャンセル返信送信: channel_id={}, original_message_id={}", 
              channel_id, original_message_id);
        
        let channel = ChannelId::from(channel_id);
        let message = CreateMessage::new()
            .content(content)
            .reference_message((ChannelId::from(channel_id), MessageId::from(original_message_id)));
        
        channel.send_message(&ctx.http, message).await?;
        
        info!("キャンセル返信送信完了");
        Ok(())
    }
}
