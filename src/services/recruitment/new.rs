use chrono::{DateTime, Utc};
use poise::serenity_prelude::ReactionType;
use poise::serenity_prelude::all::{CreateActionRow, CreateButton, CreateEmbed};
use poise::serenity_prelude::ButtonStyle;
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
    timezone: chrono_tz::Tz,
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
    let message_content = create_message_content(&quest.name, &battle_style.display_name, &expiry_date, timezone);

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
    timezone: chrono_tz::Tz,
) -> String {
    let mut message_text = format!("{}の参加者を募集します。", quest_name);

    // 攻略方法が「6属性」の場合は追加メッセージを表示
    if battle_style_name == "6属性" {
        message_text.push_str("\n参加属性を選んでください");
    }

    // 表示用にサーバー設定のタイムゾーンに変換
    let local_date = expiry_date.with_timezone(&timezone);
    message_text.push_str(&format!("\n開催日時：{}", local_date.format("%m/%d %H:%M %Z")));

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

/// ボタン版用の初期参加者一覧テキストを作成
/// 修正済みの絵文字を使用
pub fn create_initial_participants_text_for_buttons(battle_style_name: &str) -> String {
    use crate::types::{ALL_ELEMENTS_EMOJI, ELEMENT_EMOJIS, ELEMENT_NAMES, SIMPLE_JOIN_EMOJI};

    if battle_style_name == "6属性" {
        let mut text = String::new();
        for (emoji, name) in ELEMENT_EMOJIS.iter().zip(ELEMENT_NAMES.iter()) {
            text.push_str(&format!("{} {}: なし\n", emoji, name));
        }
        text.push_str(&format!("{} 全属性可能: なし\n", ALL_ELEMENTS_EMOJI));
        text
    } else {
        format!("{} 参加: なし\n", SIMPLE_JOIN_EMOJI)
    }
}

/// 募集用ボタンを作成する（ボタン版募集用）
///
/// # 引数
/// * `battle_style_name` - 攻略方法の名前（「6属性」かどうかで分岐）
///
/// # 戻り値
/// CreateActionRowのVec（Discord Message Componentsとして使用）
pub fn create_recruitment_buttons(battle_style_name: &str) -> Vec<CreateActionRow> {
    use crate::types::{ALL_ELEMENTS_EMOJI, ELEMENT_EMOJIS, ELEMENT_NAMES};

    if battle_style_name == "6属性" {
        // 6属性の場合：属性1-6ボタン + 全属性可能ボタン
        let mut element_buttons = Vec::new();
        for (i, (emoji, name)) in ELEMENT_EMOJIS.iter().zip(ELEMENT_NAMES.iter()).enumerate() {
            let button = CreateButton::new(format!("recruit_join_{}", i + 1))
                .label(format!("{} {}", emoji, name))
                .style(ButtonStyle::Primary);
            element_buttons.push(button);
        }

        // 全属性可能ボタン
        let all_elements_button = CreateButton::new("recruit_join_0")
            .label(format!("{} 全属性可能", ALL_ELEMENTS_EMOJI))
            .style(ButtonStyle::Success);

        // 全て取り消しボタン
        let leave_all_button = CreateButton::new("recruit_leave_all")
            .label("❌ 全て取り消し")
            .style(ButtonStyle::Danger);

        // 行1: 属性1-3
        let row1 = CreateActionRow::Buttons(element_buttons[0..3].to_vec());
        // 行2: 属性4-6
        let row2 = CreateActionRow::Buttons(element_buttons[3..6].to_vec());
        // 行3: 全属性可能 + 全て取り消し
        let row3 = CreateActionRow::Buttons(vec![all_elements_button, leave_all_button]);

        vec![row1, row2, row3]
    } else {
        // シンプル参加の場合：参加ボタン + 全て取り消しボタン
        use crate::types::SIMPLE_JOIN_EMOJI;

        let join_button = CreateButton::new("recruit_join")
            .label(format!("{} 参加", SIMPLE_JOIN_EMOJI))
            .style(ButtonStyle::Success);

        let leave_all_button = CreateButton::new("recruit_leave_all")
            .label("❌ 全て取り消し")
            .style(ButtonStyle::Danger);

        let row = CreateActionRow::Buttons(vec![join_button, leave_all_button]);
        vec![row]
    }
}

/// Discord操作関数（ボタン付きメッセージ送信）
pub async fn send_recruitment_message_with_buttons(
    ctx: &PoiseContext<'_>,
    recruitment_data: &RecruitmentData,
) -> types::Result<u64> {
    use poise::CreateReply;
    use poise::serenity_prelude::CreateEmbed;

    // ボタンを生成
    let buttons = create_recruitment_buttons(&recruitment_data.battle_style_name);

    // ボタン版用の初期参加者一覧を作成
    let initial_text = create_initial_participants_text_for_buttons(&recruitment_data.battle_style_name);

    // ボタン版用のembedを作成（絵文字を修正済みのものを使用）
    let embed = CreateEmbed::new()
        .title("参加者一覧")
        .description(&initial_text)
        .footer(poise::serenity_prelude::CreateEmbedFooter::new("参加者数: 0人"))
        .color(0x0099ff);

    // deferした応答を完了させる形でボタン付きメッセージを送信
    let reply = CreateReply::default()
        .content(recruitment_data.message_content.clone())
        .embed(embed)
        .components(buttons);

    let message = ctx.send(reply).await?;
    Ok(message.message().await?.id.get())
}

/// 募集データを作成する（Repository直接アクセス版）
/// Facade層から呼び出すためのラッパー関数
pub async fn create_recruitment_data_with_repos<'c, C>(
    db: &'c C,
    quest_name_or_alias: &str,
    battle_style_id: Option<i32>,
    channel_id: u64,
    guild_id: u64,
    event_date: Option<DateTime<Utc>>,
    timezone: chrono_tz::Tz,
) -> types::Result<RecruitmentData>
where
    C: sea_orm::ConnectionTrait,
{
    use crate::repository::database::battle_style_repository::SeaOrmBattleStyleRepository;
    use crate::repository::database::quest_repository::SeaOrmQuestRepository;

    let quest_repository = SeaOrmQuestRepository::new();
    let battle_style_repository = SeaOrmBattleStyleRepository::new();

    create_recruitment_data(
        db,
        &quest_repository,
        &battle_style_repository,
        quest_name_or_alias,
        battle_style_id,
        channel_id,
        guild_id,
        event_date,
        timezone,
    )
    .await
}
