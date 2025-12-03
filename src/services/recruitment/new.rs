use chrono::{DateTime, Utc};
use poise::serenity_prelude::ReactionType;
use poise::serenity_prelude::all::CreateEmbed;
use tracing::info;

use crate::models::quests::Quest;
use crate::repository::QuestRepository;
use crate::repository::database::battle_style_repository::BattleStyleRepository;
use crate::types;
use crate::types::PoiseContext;
use sea_orm::DatabaseTransaction;

/// 募集データ構造体（純粋なビジネスロジック用）
#[derive(Debug, Clone)]
pub struct RecruitmentData {
    pub quest: Quest,
    pub battle_style_id: i32,
    pub battle_style_name: String,
    pub channel_id: u64,
    pub guild_id: u64,
    pub expiry_date: DateTime<chrono::Utc>,
    pub message_content: String,
    pub embed: CreateEmbed,
    pub reactions: Vec<poise::serenity_prelude::ReactionType>,
}

/// 募集データを作成する（QuestRepository, BattleStyleRepositoryを使用）
pub async fn create_recruitment_data<'c, C, Q, B>(
    db: &'c C,
    quest_repository: &Q,
    battle_style_repository: &B,
    quest_name_or_alias: &str,
    battle_style_id: Option<i32>,
    channel_id: u64,
    guild_id: u64,
    event_date: Option<DateTime<Utc>>,
) -> types::Result<RecruitmentData>
where
    C: sea_orm::ConnectionTrait,
    Q: QuestRepository,
    B: BattleStyleRepository,
{
    // クエスト名またはエイリアスで検索
    let search_results = quest_repository
        .search_by_name_or_alias(db, quest_name_or_alias)
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
        .get_by_target_id(db, quest_search_result.quest_id)
        .await?
        .ok_or_else(|| types::AppError::NotFound(format!(
            "クエストID {} の詳細情報が見つかりませんでした",
            quest_search_result.quest_id
        )))?;

    // イベント日時の決定（既にUTCで受け取っている）
    let expiry_date = event_date.unwrap_or_else(|| chrono::Utc::now() + chrono::Duration::days(7));

    // battle_style_idの決定：パラメータで指定されていればそれを使用、未指定ならquestのdefault_battle_style_idを使用
    let actual_battle_style_id = battle_style_id.unwrap_or(quest.default_battle_style_id);

    // battle_stylesテーブルから攻略方法の詳細を取得
    let battle_style = battle_style_repository
        .get_by_id(db, actual_battle_style_id)
        .await?
        .ok_or_else(|| types::AppError::NotFound(format!(
            "攻略方法ID {} が見つかりませんでした",
            actual_battle_style_id
        )))?;

    // reactionsをパース
    let reactions = parse_reactions(battle_style.reactions.as_deref().unwrap_or("✅"));

    // メッセージ内容を作成
    let message_content = create_message_content(&quest.name, &battle_style.display_name, &expiry_date);

    // 初期参加者一覧を作成
    let initial_participants_text = create_initial_participants_text(&reactions);

    Ok(RecruitmentData {
        quest,
        battle_style_id: actual_battle_style_id,
        battle_style_name: battle_style.display_name,
        channel_id,
        guild_id,
        expiry_date,
        message_content,
        embed: poise::serenity_prelude::CreateEmbed::new()
            .title("参加者一覧")
            .description(&initial_participants_text)
            .color(0x0099ff),
        reactions,
    })
}

/// Discord操作関数（メッセージ送信）
pub async fn send_recruitment_message(
    ctx: &PoiseContext<'_>,
    recruitment_data: &RecruitmentData,
) -> types::Result<u64> {
    use poise::CreateReply;

    // deferした応答を完了させる形でメッセージを送信
    let reply = CreateReply::default()
        .content(recruitment_data.message_content.clone())
        .embed(recruitment_data.embed.clone());

    let message = ctx.send(reply).await?;
    Ok(message.message().await?.id.get())
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
pub async fn save_recruitment<R: crate::repository::BattleRecruitmentsRepository>(
    txn: &DatabaseTransaction,
    battle_recruitment_repo: &R,
    recruitment_data: &RecruitmentData,
    message_id: u64,
) -> types::Result<crate::models::battle_recruitments::BattleRecruitments> {
    // battle_style_idは実際に使用されたものを保存
    let recruitment = battle_recruitment_repo
        .create_with_txn(
            txn,
            recruitment_data.guild_id,
            recruitment_data.channel_id,
            message_id,
            recruitment_data.quest.id,
            recruitment_data.battle_style_id,
            recruitment_data.expiry_date,
        )
        .await?;

    info!("Successfully registered recruitment in database");
    Ok(recruitment)
}

/// メッセージ内容を作成する（純粋関数）
pub fn create_message_content(
    quest_name: &str,
    battle_style_name: &str,
    expiry_date: &DateTime<chrono::Utc>,
) -> String {
    let mut message_text = format!("{}の参加者を募集します。", quest_name);

    // 攻略方法が「6属性」の場合は追加メッセージを表示
    if battle_style_name == "6属性" {
        message_text.push_str("\n参加属性を選んでください");
    }

    // 表示用にJST変換（UTC+9）
    let jst_date = *expiry_date + chrono::Duration::hours(9);
    message_text.push_str(&format!("\n開催日時：{} (JST)", jst_date.format("%m/%d %H:%M")));

    message_text
}

/// reactionsをパースする（カンマ区切りの絵文字文字列をReactionTypeのVecに変換）
fn parse_reactions(reactions_str: &str) -> Vec<ReactionType> {
    reactions_str
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|emoji| ReactionType::Unicode(emoji.to_string()))
        .collect()
}

/// 初期参加者一覧テキストを作成
/// すべてのリアクション絵文字を「なし」で表示
fn create_initial_participants_text(reactions: &[ReactionType]) -> String {
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
