use chrono::{DateTime, Duration, Local};
use poise::serenity_prelude::all::CreateEmbed;
use std::sync::Arc;
use tracing::{error, info};

use crate::models::quest::Quest;
use crate::repository::{BattleRecruitmentRepository, QuestRepository};
use crate::types::{battle_type::BattleType, DiscordOperation};

/// 募集データ構造体（純粋なビジネスロジック用）
#[derive(Debug, Clone)]
pub struct RecruitmentData {
    pub quest: Quest,
    pub battle_type: BattleType,
    pub channel_id: u64,
    pub guild_id: u64,
    pub expiry_date: DateTime<chrono::Utc>,
    pub message_content: String,
    pub embed: CreateEmbed,
    pub reactions: Vec<poise::serenity_prelude::ReactionType>,
}

pub(crate) struct NewParameter {
    pub guild_id: i64,
    pub channel_id: i64,
    pub message_id: i64,
    pub quest: Quest,
    pub target_id: i32,
    pub battle_type_id: i32,
    pub expiry_date: chrono::DateTime<chrono::Utc>,
}
pub struct NewRecruitmentService {
    battle_recruitment_repo: Arc<dyn BattleRecruitmentRepository>,
    quest_repo: Arc<dyn QuestRepository>,
}

impl NewRecruitmentService {
    /// 依存性注入パターンに従ったコンストラクタ
    pub fn new(
        battle_recruitment_repo: Arc<dyn BattleRecruitmentRepository>,
        quest_repo: Arc<dyn QuestRepository>,
    ) -> Self {
        Self {
            battle_recruitment_repo,
            quest_repo,
        }
    }

    /// 募集データを作成する（純粋なビジネスロジック）
    pub async fn create_recruitment_data(
        &self,
        quest_alias: &str,
        battle_type: BattleType,
        channel_id: u64,
        guild_id: u64,
        event_date: Option<DateTime<Local>>,
    ) -> Result<RecruitmentData, String> {
        // 1. クエストを取得
        let quest = self.get_quest_by_alias(quest_alias).await?;

        // 2. イベント日時を決定（指定されていない場合はデフォルト）
        let expiry_date = event_date
            .unwrap_or_else(|| Local::now() + Duration::days(7))
            .with_timezone(&chrono::Utc);

        // 3. メッセージ内容を作成
        let message_content = self.create_message_content(&quest.quest_name, &battle_type, &expiry_date);

        // 4. 埋め込みメッセージを作成
        let embed = CreateEmbed::new()
            .title("参加者一覧")
            .description("現在参加者はいません。")
            .color(0x0099ff);

        // 5. リアクション一覧を取得
        let reactions = battle_type.reactions();

        Ok(RecruitmentData {
            quest,
            battle_type,
            channel_id,
            guild_id,
            expiry_date,
            message_content,
            embed,
            reactions,
        })
    }

    /// データベースに募集を保存する（純粋なビジネスロジック）
    pub async fn save_recruitment(
        &self,
        recruitment_data: &RecruitmentData,
        message_id: u64,
    ) -> Result<(), String> {
        self.register_recruitment(
            recruitment_data.guild_id as i64,
            recruitment_data.channel_id as i64,
            message_id as i64,
            recruitment_data.quest.target_id,
            recruitment_data.battle_type.clone(),
            recruitment_data.expiry_date.with_timezone(&Local),
        )
            .await
    }

    /// メッセージ内容を作成する（純粋関数）
    fn create_message_content(
        &self,
        quest_name: &str,
        battle_type: &BattleType,
        expiry_date: &DateTime<chrono::Utc>,
    ) -> String {
        let mut message_text = format!("{}の参加者を募集します。", quest_name);

        if *battle_type == BattleType::AllElement {
            message_text.push_str("\n参加属性を選んでください");
        }

        let local_date = expiry_date.with_timezone(&Local);
        message_text.push_str(&format!("\n開催日時：{}", local_date.format("%m/%d %H:%M")));

        message_text
    }

    /// クエストエイリアスからクエスト情報を取得
    async fn get_quest_by_alias(&self, alias: &str) -> Result<Quest, String> {
        match self.quest_repo.get_by_alias(alias).await {
            Ok(Some(quest)) => Ok(quest),
            Ok(None) => Err(format!("Quest not found for alias: {}", alias)),
            Err(e) => {
                error!("Database error when getting quest by alias: {:?}", e);
                Err(format!("Database error: {}", e))
            }
        }
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
        match self
            .battle_recruitment_repo
            .create(
                guild_id,
                channel_id,
                message_id,
                target_id,
                battle_type as i32,
                expiry_date.with_timezone(&chrono::Utc),
            )
            .await
        {
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
