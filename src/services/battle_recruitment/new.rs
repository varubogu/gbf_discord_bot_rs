use chrono::{DateTime, Duration, Local};
use poise::serenity_prelude::ReactionType;
use poise::serenity_prelude::all::CreateEmbed;
use tracing::{error, info};

use crate::infrastructure::database::container::RepositoryContainer;
use crate::models::quests::Quest;
use crate::types;
use crate::types::battle_type::BattleType;
use crate::types::{AppError, AppState, PoiseContext};
use sea_orm::DatabaseTransaction;

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

/// 募集データを作成する（純粋なビジネスロジック）
pub async fn create_recruitment_data(
    quest_alias: &str,
    battle_type: BattleType,
    channel_id: u64,
    guild_id: u64,
    app_state: &AppState,
    event_date: Option<DateTime<Local>>,
) -> types::Result<RecruitmentData> {
    // 簡略版ヘルパー関数を使用（既存のFacadeとの互換性を保つため）
    let mut recruitment_data =
        create_recruitment_data_simple(quest_alias, battle_type, channel_id, guild_id);

    // イベント日時を指定されたものに更新（指定されている場合）
    if let Some(event_date) = event_date {
        recruitment_data.expiry_date = event_date.with_timezone(&chrono::Utc);
        recruitment_data.message_content = create_message_content(
            &recruitment_data.quest.name,
            &battle_type,
            &recruitment_data.expiry_date,
        );
    }

    Ok(recruitment_data)
}

/// Discord操作関数（メッセージ送信）
pub async fn send_recruitment_message(
    ctx: &PoiseContext<'_>,
    recruitment_data: &RecruitmentData,
) -> types::Result<u64> {
    let recruit_message = ctx.say(recruitment_data.message_content.clone()).await?;
    let message = recruit_message.message().await?;
    Ok(message.id.get())
}

/// Discord操作関数（リアクション追加）
pub async fn add_recruitment_reactions(
    ctx: &PoiseContext<'_>,
    message_id: u64,
    reactions: &[ReactionType],
) -> types::Result<()> {
    let message_id = poise::serenity_prelude::MessageId::new(message_id);
    let message = ctx.channel_id().message(&ctx.http(), message_id).await?;

    for reaction in reactions {
        message.react(&ctx.http(), reaction.clone()).await?;
    }
    Ok(())
}

/// データ保存関数
pub async fn save_recruitment(
    recruitment_data: &RecruitmentData,
    message_id: u64,
    txn: &DatabaseTransaction,
    app_state: &AppState,
) -> types::Result<()> {
    let repos = RepositoryContainer::new(&app_state.db_connection);
    let battle_recruitment_repo = repos.battle_recruitment();

    battle_recruitment_repo
        .create_with_txn(
            txn,
            recruitment_data.guild_id as i64,
            recruitment_data.channel_id as i64,
            message_id as i64,
            recruitment_data.quest.id,
            recruitment_data.battle_type as i32,
            recruitment_data.expiry_date,
        )
        .await?;

    info!("Successfully registered recruitment in database");
    Ok(())
}

/// メッセージ内容を作成する（純粋関数）
fn create_message_content(
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

/// 募集データ作成のヘルパー関数（簡略版）
pub fn create_recruitment_data_simple(
    quest_alias: &str,
    battle_type: BattleType,
    channel_id: u64,
    guild_id: u64,
) -> RecruitmentData {
    RecruitmentData {
        quest: crate::models::quests::Quest {
            id: 1,
            name: quest_alias.to_string(),
            default_battle_style: 1,
            recruit_count: 6,
            available_battle_styles: 1,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
        battle_type,
        channel_id,
        guild_id,
        expiry_date: chrono::Utc::now() + chrono::Duration::days(7),
        message_content: format!("{}の参加者を募集します。", quest_alias),
        embed: poise::serenity_prelude::CreateEmbed::new()
            .title("参加者一覧")
            .description("現在参加者はいません。")
            .color(0x0099ff),
        reactions: battle_type.reactions(),
    }
}
