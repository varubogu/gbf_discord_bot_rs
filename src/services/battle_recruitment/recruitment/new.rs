use std::sync::Arc;
use chrono::{DateTime, Local, Duration};
use poise::serenity_prelude::all::{Context, CreateEmbed, CreateMessage, Message, ChannelId};
use tracing::{error, info};

use crate::repository::Database;
use crate::models::quest::Quest;
use crate::types::BattleType;

pub struct NewRecruitmentService {
    db: Arc<Database>,
}

impl NewRecruitmentService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// 新規募集を作成する
    /// Python版のbase_battle_recruiment_cog.py の recruitment() メソッドに相当
    pub async fn create_recruitment(
        &self,
        ctx: &Context,
        channel_id: u64,
        guild_id: u64,
        quest_alias: &str,
        battle_type: BattleType,
        event_date: Option<DateTime<Local>>,
    ) -> Result<Message, String> {
        // 1. クエストを取得
        let quest = self.get_quest_by_alias(quest_alias).await?;
        
        // 2. イベント日時を決定（指定されていない場合はデフォルト）
        let expiry_date = event_date.unwrap_or_else(|| {
            Local::now() + Duration::days(7)
        });

        // 3. 募集メッセージを作成・送信
        let message = self.send_recruitment_message(
            ctx,
            channel_id,
            &quest.quest_name,
            battle_type.clone(),
            expiry_date,
        ).await?;

        // 4. リアクションを追加
        self.add_reactions(ctx, &message, battle_type.clone()).await?;

        // 5. データベースに登録
        self.register_recruitment(
            guild_id as i64,
            channel_id as i64,
            message.id.get() as i64,
            quest.target_id,
            battle_type,
            expiry_date,
        ).await?;

        info!("Successfully created recruitment for quest: {}", quest.quest_name);
        Ok(message)
    }

    /// クエストエイリアスからクエスト情報を取得
    async fn get_quest_by_alias(&self, alias: &str) -> Result<Quest, String> {
        match self.db.quest.get_by_alias(alias).await {
            Ok(Some(quest)) => Ok(quest),
            Ok(None) => Err(format!("Quest not found for alias: {}", alias)),
            Err(e) => {
                error!("Database error when getting quest by alias: {:?}", e);
                Err(format!("Database error: {}", e))
            }
        }
    }

    /// 募集メッセージを作成・送信
    /// Python版の _send_message() に相当
    async fn send_recruitment_message(
        &self,
        ctx: &Context,
        channel_id: u64,
        quest_name: &str,
        battle_type: BattleType,
        event_date: DateTime<Local>,
    ) -> Result<Message, String> {
        // メッセージテキストを作成
        let mut message_text = format!("{}の参加者を募集します。", quest_name);
        
        if battle_type == BattleType::AllElement {
            message_text.push_str("\n参加属性を選んでください");
        }

        message_text.push_str(&format!(
            "\n開催日時：{}",
            event_date.format("%m/%d %H:%M")
        ));

        // 埋め込みメッセージを作成
        let embed = CreateEmbed::new()
            .title("参加者一覧")
            .description("現在参加者はいません。")
            .color(0x0099ff);

        // メッセージを送信
        let builder = CreateMessage::new()
            .content(message_text)
            .embed(embed);

        match ChannelId::from(channel_id).send_message(&ctx.http, builder).await {
            Ok(message) => Ok(message),
            Err(e) => {
                error!("Error sending recruitment message: {:?}", e);
                Err(format!("Failed to send message: {}", e))
            }
        }
    }

    /// メッセージにリアクションを追加
    /// Python版の _add_reaction() に相当
    async fn add_reactions(
        &self,
        ctx: &Context,
        message: &Message,
        battle_type: BattleType,
    ) -> Result<(), String> {
        for reaction in battle_type.reactions() {
            let reaction_clone = reaction.clone();
            if let Err(e) = message.react(&ctx.http, reaction).await {
                error!("Error adding reaction {:?}: {:?}", reaction_clone, e);
                // 一つのリアクション失敗でも処理を継続
            }
        }
        Ok(())
    }

    /// 募集情報をデータベースに登録
    /// Python版の _regist() に相当
    async fn register_recruitment(
        &self,
        guild_id: i64,
        channel_id: i64,
        message_id: i64,
        target_id: i32,
        battle_type: BattleType,
        expiry_date: DateTime<Local>,
    ) -> Result<(), String> {
        match self.db.battle_recruitment.create(
            guild_id,
            channel_id,
            message_id,
            target_id,
            battle_type as i32,
            expiry_date.with_timezone(&chrono::Utc),
        ).await {
            Ok(_) => {
                info!("Successfully registered recruitment in database");
                Ok(())
            }
            Err(e) => {
                error!("Error registering recruitment: {:?}", e);
                Err(format!("Failed to register recruitment: {}", e))
            }
        }
    }
}