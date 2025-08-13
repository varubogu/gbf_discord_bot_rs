use std::sync::Arc;
use chrono::{DateTime, Local};
use poise::serenity_prelude::all::{Context, Message, ChannelId, EditMessage, CreateEmbed};
use tracing::{error, info, warn};

use crate::repository::Database;
use crate::models::battle_recruitment::BattleRecruitment;
use crate::types::BattleType;

pub struct UpdateRecruitmentService {
    db: Arc<Database>,
}

impl UpdateRecruitmentService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// 募集メッセージの内容を更新する
    pub async fn update_recruitment_message(
        &self,
        ctx: &Context,
        guild_id: u64,
        channel_id: u64,
        message_id: u64,
        new_content: Option<String>,
        new_embed: Option<CreateEmbed>,
    ) -> Result<(), String> {
        // 募集が存在するかチェック
        let _recruitment = match self.db.get_battle_recruitment(
            guild_id as i64,
            channel_id as i64,
            message_id as i64,
        ).await {
            Ok(Some(recruitment)) => recruitment,
            Ok(None) => {
                warn!("Recruitment not found for message: {}", message_id);
                return Err("募集が見つかりませんでした。".to_string());
            },
            Err(e) => {
                error!("Error fetching recruitment: {:?}", e);
                return Err("データベースエラーが発生しました。".to_string());
            }
        };

        // メッセージを更新
        let channel = ChannelId::from(channel_id);
        let mut edit_builder = EditMessage::new();

        if let Some(content) = new_content {
            edit_builder = edit_builder.content(content);
        }

        if let Some(embed) = new_embed {
            edit_builder = edit_builder.embed(embed);
        }

        match channel.edit_message(&ctx.http, message_id, edit_builder).await {
            Ok(_) => {
                info!("Successfully updated recruitment message: {}", message_id);
                Ok(())
            },
            Err(e) => {
                error!("Failed to update message: {:?}", e);
                Err("メッセージの更新に失敗しました。".to_string())
            }
        }
    }

    /// 募集の参加者リスト埋め込みを更新する
    pub async fn update_participants_embed(
        &self,
        ctx: &Context,
        guild_id: u64,
        channel_id: u64,
        message_id: u64,
        participant_count: usize,
        capacity: usize,
    ) -> Result<(), String> {
        let embed = CreateEmbed::new()
            .title("参加者一覧")
            .description(format!(
                "現在の参加者: {}/{}",
                participant_count,
                capacity
            ))
            .color(if participant_count >= capacity { 0x00ff00 } else { 0x0099ff });

        self.update_recruitment_message(
            ctx,
            guild_id,
            channel_id,
            message_id,
            None,
            Some(embed),
        ).await
    }

    /// 募集の開催日時を更新する
    pub async fn update_recruitment_date(
        &self,
        ctx: &Context,
        guild_id: u64,
        channel_id: u64,
        message_id: u64,
        new_date: DateTime<Local>,
    ) -> Result<(), String> {
        // 募集情報を取得
        let recruitment = match self.db.get_battle_recruitment(
            guild_id as i64,
            channel_id as i64,
            message_id as i64,
        ).await {
            Ok(Some(recruitment)) => recruitment,
            Ok(None) => {
                return Err("募集が見つかりませんでした。".to_string());
            },
            Err(e) => {
                error!("Error fetching recruitment: {:?}", e);
                return Err("データベースエラーが発生しました。".to_string());
            }
        };

        // 注意: 実際の実装ではクエスト名を取得する必要がある
        let quest_name = "クエスト"; // 簡易版実装

        // 新しいメッセージ内容を作成
        let new_content = format!(
            "{}の参加者を募集します。\n開催日時：{}",
            quest_name,
            new_date.format("%m/%d %H:%M")
        );

        self.update_recruitment_message(
            ctx,
            guild_id,
            channel_id,
            message_id,
            Some(new_content),
            None,
        ).await?;

        info!("Updated recruitment date for message: {}", message_id);
        Ok(())
    }

    /// 募集メッセージにステータス更新を追加
    pub async fn add_status_update(
        &self,
        ctx: &Context,
        channel_id: u64,
        original_message_id: u64,
        status: &str,
    ) -> Result<Message, String> {
        let status_message = format!(
            "募集更新 (元メッセージ: {}): {}",
            original_message_id,
            status
        );

        match ChannelId::from(channel_id).say(&ctx.http, status_message).await {
            Ok(message) => {
                info!("Added status update for recruitment: {}", original_message_id);
                Ok(message)
            },
            Err(e) => {
                error!("Failed to send status update: {:?}", e);
                Err("ステータス更新の送信に失敗しました。".to_string())
            }
        }
    }

    /// 募集完了時の最終更新
    pub async fn mark_recruitment_complete(
        &self,
        ctx: &Context,
        guild_id: u64,
        channel_id: u64,
        message_id: u64,
        participants: Vec<String>, // ユーザーメンション
    ) -> Result<(), String> {
        let embed = CreateEmbed::new()
            .title("募集完了")
            .description("メンバーが揃いました！")
            .field("参加者", participants.join("\n"), false)
            .color(0x00ff00)
            .timestamp(chrono::Utc::now());

        self.update_recruitment_message(
            ctx,
            guild_id,
            channel_id,
            message_id,
            Some("✅ 募集完了".to_string()),
            Some(embed),
        ).await?;

        // 完了通知メッセージを送信
        let completion_message = format!(
            "{}\nメンバーが揃いました！",
            participants.join(" ")
        );

        match ChannelId::from(channel_id).say(&ctx.http, completion_message).await {
            Ok(_) => {
                info!("Marked recruitment as complete: {}", message_id);
                Ok(())
            },
            Err(e) => {
                error!("Failed to send completion message: {:?}", e);
                // メッセージ更新は成功したので、エラーとせずログのみ
                info!("Message update succeeded but completion notification failed");
                Ok(())
            }
        }
    }

    /// 募集の緊急更新（重要な変更時）
    pub async fn urgent_update(
        &self,
        ctx: &Context,
        channel_id: u64,
        message_id: u64,
        urgent_message: &str,
    ) -> Result<(), String> {
        let embed = CreateEmbed::new()
            .title("🚨 重要な更新")
            .description(urgent_message)
            .color(0xff0000)
            .timestamp(chrono::Utc::now());

        // 元のメッセージに緊急マークを追加
        let edit_builder = EditMessage::new()
            .content("🚨 重要更新あり 🚨")
            .embed(embed);

        match ChannelId::from(channel_id).edit_message(&ctx.http, message_id, edit_builder).await {
            Ok(_) => {
                info!("Applied urgent update to message: {}", message_id);
                Ok(())
            },
            Err(e) => {
                error!("Failed to apply urgent update: {:?}", e);
                Err("緊急更新の適用に失敗しました。".to_string())
            }
        }
    }
}