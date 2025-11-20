use chrono::{DateTime, Local};
use poise::serenity_prelude::ReactionType;
use poise::serenity_prelude::all::CreateEmbed;
use tracing::info;

use crate::models::quests::Quest;
use crate::repository::QuestRepository;
use crate::types;
use crate::types::PoiseContext;
use crate::types::battle_type::BattleType;
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

/// 募集データを作成する（QuestRepositoryを使用）
pub async fn create_recruitment_data(
    quest_repository: &dyn QuestRepository,
    quest_name_or_alias: &str,
    battle_type: BattleType,
    channel_id: u64,
    guild_id: u64,
    event_date: Option<DateTime<Local>>,
) -> types::Result<RecruitmentData> {
    // クエスト名またはエイリアスで検索
    let search_results = quest_repository
        .search_by_name_or_alias(quest_name_or_alias)
        .await?;

    // 最初にマッチしたクエストを使用
    let quest_search_result = search_results
        .first()
        .ok_or_else(|| types::AppError::NotFound(format!(
            "クエスト '{}' が見つかりませんでした",
            quest_name_or_alias
        )))?;

    // クエストの詳細情報を取得
    let quest = quest_repository
        .get_by_target_id(quest_search_result.quest_id)
        .await?
        .ok_or_else(|| types::AppError::NotFound(format!(
            "クエストID {} の詳細情報が見つかりませんでした",
            quest_search_result.quest_id
        )))?;

    // イベント日時の決定
    let expiry_date = if let Some(event_date) = event_date {
        event_date.with_timezone(&chrono::Utc)
    } else {
        chrono::Utc::now() + chrono::Duration::days(7)
    };

    // questのdefault_battle_styleからBattleTypeを決定
    let actual_battle_type = BattleType::from_value(quest.default_battle_style)
        .unwrap_or(battle_type);

    // メッセージ内容を作成
    let message_content = create_message_content(&quest.name, &actual_battle_type, &expiry_date);

    // BattleTypeに応じた初期参加者一覧を作成
    let initial_participants_text = create_initial_participants_text(&actual_battle_type);

    Ok(RecruitmentData {
        quest,
        battle_type: actual_battle_type,
        channel_id,
        guild_id,
        expiry_date,
        message_content,
        embed: poise::serenity_prelude::CreateEmbed::new()
            .title("参加者一覧")
            .description(&initial_participants_text)
            .color(0x0099ff),
        reactions: actual_battle_type.reactions(),
    })
}

/// Discord操作関数（メッセージ送信）
pub async fn send_recruitment_message(
    ctx: &PoiseContext<'_>,
    recruitment_data: &RecruitmentData,
) -> types::Result<u64> {
    use poise::serenity_prelude::all::CreateMessage;

    // メッセージにEmbedを含めて送信
    let builder = CreateMessage::new()
        .content(recruitment_data.message_content.clone())
        .embed(recruitment_data.embed.clone());

    let message = ctx.channel_id().send_message(ctx.http(), builder).await?;
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
    txn: &DatabaseTransaction,
    battle_recruitment_repo: &dyn crate::repository::BattleRecruitmentsRepository,
    recruitment_data: &RecruitmentData,
    message_id: u64,
) -> types::Result<()> {
    // battle_type_idはquestsテーブルのdefault_battle_styleを使用
    battle_recruitment_repo
        .create_with_txn(
            txn,
            recruitment_data.guild_id,
            recruitment_data.channel_id,
            message_id,
            recruitment_data.quest.id,
            recruitment_data.quest.default_battle_style,
            recruitment_data.expiry_date,
        )
        .await?;

    info!("Successfully registered recruitment in database");
    Ok(())
}

/// メッセージ内容を作成する（純粋関数）
pub fn create_message_content(
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

/// BattleTypeに応じた初期参加者一覧テキストを作成
/// すべてのリアクション絵文字を「なし」で表示
fn create_initial_participants_text(battle_type: &BattleType) -> String {
    let reactions = battle_type.reactions();
    let mut text = String::new();

    for reaction in reactions {
        // ReactionTypeから絵文字文字列を取得
        let emoji = match reaction {
            ReactionType::Unicode(emoji_str) => emoji_str,
            _ => continue,
        };
        text.push_str(&format!("{} なし\n", emoji));
    }

    if text.is_empty() {
        "現在参加者はいません。".to_string()
    } else {
        text
    }
}
