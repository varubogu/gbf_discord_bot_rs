use chrono::{DateTime, Utc};
use poise::serenity_prelude::ButtonStyle;
use poise::serenity_prelude::ReactionType;
use poise::serenity_prelude::all::{
    CreateActionRow, CreateButton, CreateEmbed, CreateSelectMenu, CreateSelectMenuKind,
    CreateSelectMenuOption,
};
use std::collections::HashMap;
use tracing::info;

use crate::models::quests::Quest;
use crate::repository::QuestRepository;
use crate::repository::database::battle_style_repository::BattleStyleRepository;
use crate::services::guild_environment_service::ElementEmojis;
use crate::services::message::{MessageService, MessageTextId};
use crate::services::unified_datetime_parser::ParsedDismissalTime;
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
    pub element_emojis: ElementEmojis,
}

/// 募集作成パラメータ
pub struct RecruitmentParams<'a> {
    pub quest_name_or_alias: &'a str,
    pub battle_style_id: Option<i32>,
    pub channel_id: u64,
    pub guild_id: u64,
    pub event_date: Option<DateTime<Utc>>,
    pub timezone: chrono_tz::Tz,
}

/// 募集データを作成する（QuestRepository, BattleStyleRepositoryを使用）
pub async fn create_recruitment_data<C, Q, B>(
    db: &C,
    quest_repository: &Q,
    battle_style_repository: &B,
    element_emojis: &ElementEmojis,
    params: RecruitmentParams<'_>,
) -> types::Result<RecruitmentData>
where
    C: sea_orm::ConnectionTrait,
    Q: QuestRepository,
    B: BattleStyleRepository,
{
    // クエスト名またはエイリアスで検索
    let search_results = quest_repository
        .search_by_name_or_alias(db, params.quest_name_or_alias)
        .await?;

    // 最初にマッチしたクエストを使用
    let quest_search_result = search_results.first().ok_or_else(|| {
        types::AppError::NotFound(format!(
            "クエスト '{}' が見つかりませんでした",
            params.quest_name_or_alias
        ))
    })?;

    // クエストの詳細情報を取得
    let quest = quest_repository
        .get_by_target_id(db, quest_search_result.quest_id)
        .await?
        .ok_or_else(|| {
            types::AppError::NotFound(format!(
                "クエストID {} の詳細情報が見つかりませんでした",
                quest_search_result.quest_id
            ))
        })?;

    // イベント日時の決定（既にUTCで受け取っている）
    let expiry_date = params
        .event_date
        .unwrap_or_else(|| chrono::Utc::now() + chrono::Duration::days(7));

    // battle_style_idの決定：パラメータで指定されていればそれを使用、未指定ならquestのdefault_battle_style_idを使用
    let actual_battle_style_id = params
        .battle_style_id
        .unwrap_or(quest.default_battle_style_id);

    // battle_stylesテーブルから攻略方法の詳細を取得
    let battle_style = battle_style_repository
        .get_by_id(db, actual_battle_style_id)
        .await?
        .ok_or_else(|| {
            types::AppError::NotFound(format!(
                "攻略方法ID {actual_battle_style_id} が見つかりませんでした"
            ))
        })?;

    // reactionsをパース
    // 6属性の場合はelement_emojisから取得、それ以外はbattle_styleのreactionsをパース
    let reactions = if battle_style.display_name == "6属性" {
        let emojis_array = element_emojis.as_array();
        emojis_array
            .iter()
            .map(|emoji| ReactionType::Unicode(emoji.to_string()))
            .collect()
    } else {
        parse_reactions(battle_style.reactions.as_deref().unwrap_or("✅"))
    };

    // メッセージ内容を作成（解散時刻なし - create_recruitment_dataでは解散時刻情報がないため）
    let message_content = create_message_content(
        db,
        &quest.name,
        &battle_style.display_name,
        &expiry_date,
        params.timezone,
        Some(params.guild_id as i64),
        None,
    )
    .await?;

    // 初期参加者一覧を作成
    let initial_participants_text =
        create_initial_participants_text(db, &reactions, Some(params.guild_id as i64)).await?;

    Ok(RecruitmentData {
        quest,
        battle_style_id: actual_battle_style_id,
        battle_style_name: battle_style.display_name,
        channel_id: params.channel_id,
        guild_id: params.guild_id,
        expiry_date,
        message_content,
        embed: poise::serenity_prelude::CreateEmbed::new()
            .title("参加者一覧")
            .description(&initial_participants_text)
            .color(0x0099ff),
        reactions,
        element_emojis: element_emojis.clone(),
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
            crate::repository::CreateBattleRecruitmentParams {
                guild_id: recruitment_data.guild_id,
                channel_id: recruitment_data.channel_id,
                message_id,
                quest_id: recruitment_data.quest.id,
                battle_style_id: recruitment_data.battle_style_id,
                quest_start_at: recruitment_data.expiry_date,
            },
        )
        .await?;

    info!("Successfully registered recruitment in database");
    Ok(recruitment)
}

/// メッセージ内容を作成する（メッセージサービス使用版）
pub async fn create_message_content<C>(
    db: &C,
    quest_name: &str,
    battle_style_name: &str,
    expiry_date: &DateTime<chrono::Utc>,
    timezone: chrono_tz::Tz,
    guild_id: Option<i64>,
    dismissal_times: Option<&[ParsedDismissalTime]>,
) -> types::Result<String>
where
    C: sea_orm::ConnectionTrait,
{
    let message_service = MessageService::new();

    // メッセージIDを決定
    let message_id = if battle_style_name == "6属性" {
        MessageTextId::RecruitmentDisplaySixElements
    } else {
        MessageTextId::RecruitmentDisplayNormal
    };

    // パラメータを準備
    let mut params = HashMap::new();
    params.insert("quest_name".to_string(), quest_name.to_string());

    // 基本メッセージを取得
    let mut message_text = message_service
        .get_message(db, message_id.as_str(), params, guild_id, Some("ja"))
        .await?;

    // 開催日時を追加
    let local_date = expiry_date.with_timezone(&timezone);

    // 日時ラベルとフォーマットを取得
    let date_label = message_service
        .get_message(
            db,
            MessageTextId::RecruitmentDisplayEventDateLabel.as_str(),
            HashMap::new(),
            guild_id,
            Some("ja"),
        )
        .await?;

    let date_format = message_service
        .get_message(
            db,
            MessageTextId::RecruitmentDisplayDateFormat.as_str(),
            HashMap::new(),
            guild_id,
            Some("ja"),
        )
        .await?;

    message_text.push_str(&format!(
        "\n{}{}",
        date_label,
        local_date.format(&date_format)
    ));

    // 解散時刻を追加
    if let Some(dismissal_times_list) = dismissal_times {
        if !dismissal_times_list.is_empty() {
            let dismissal_label = message_service
                .get_message(
                    db,
                    MessageTextId::RecruitmentDisplayDismissalTimesLabel.as_str(),
                    HashMap::new(),
                    guild_id,
                    Some("ja"),
                )
                .await?;

            let dismissal_texts: Vec<String> = dismissal_times_list
                .iter()
                .map(|dt| format_dismissal_time(dt, expiry_date, &timezone, &date_format))
                .collect();

            message_text.push_str(&format!(
                "\n{}{}",
                dismissal_label,
                dismissal_texts.join(", ")
            ));
        }
    }

    Ok(message_text)
}

/// 解散時刻をフォーマット（相対時刻と絶対時刻の両方を表示）
fn format_dismissal_time(
    dismissal_time: &ParsedDismissalTime,
    departure_time: &DateTime<Utc>,
    timezone: &chrono_tz::Tz,
    date_format: &str,
) -> String {
    match dismissal_time {
        ParsedDismissalTime::Absolute {
            input_value,
            datetime,
        } => {
            let local_datetime = datetime.with_timezone(timezone);
            let formatted_datetime = local_datetime.format(date_format).to_string();
            format!("{input_value} ({formatted_datetime})")
        }
        ParsedDismissalTime::Relative {
            input_value,
            days,
            hours,
            minutes,
        } => {
            use chrono::Duration;
            let duration = Duration::days(*days as i64)
                + Duration::hours(*hours as i64)
                + Duration::minutes(*minutes as i64);
            let dismissal_datetime = *departure_time - duration;
            let local_datetime = dismissal_datetime.with_timezone(timezone);
            let formatted_datetime = local_datetime.format(date_format).to_string();
            format!("{input_value} ({formatted_datetime})")
        }
    }
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
/// すべてのリアクション絵文字を「なし」で表示（メッセージサービス使用版）
async fn create_initial_participants_text<C>(
    db: &C,
    reactions: &[ReactionType],
    guild_id: Option<i64>,
) -> types::Result<String>
where
    C: sea_orm::ConnectionTrait,
{
    let message_service = MessageService::new();
    let mut text = String::new();

    // 「なし」テキストを取得
    let no_participants = message_service
        .get_message(
            db,
            MessageTextId::RecruitmentDisplayNoParticipants.as_str(),
            HashMap::new(),
            guild_id,
            Some("ja"),
        )
        .await?;

    for reaction in reactions {
        // ReactionTypeから絵文字文字列を取得
        let emoji = match reaction {
            ReactionType::Unicode(emoji_str) => emoji_str,
            _ => continue,
        };
        text.push_str(&format!("{emoji} {no_participants}\n"));
    }

    if text.is_empty() {
        Ok("現在参加者はいません。".to_string())
    } else {
        Ok(text)
    }
}

/// ボタン版用の初期参加者一覧テキストを作成
/// 修正済みの絵文字を使用
pub fn create_initial_participants_text_for_buttons(
    battle_style_name: &str,
    element_emojis: &ElementEmojis,
) -> String {
    use crate::types::{ALL_ELEMENTS_EMOJI, ELEMENT_NAMES, SIMPLE_JOIN_EMOJI};

    if battle_style_name == "6属性" {
        let mut text = String::new();
        let emojis_array = element_emojis.as_array();
        for (emoji, name) in emojis_array.iter().zip(ELEMENT_NAMES.iter()) {
            text.push_str(&format!("{emoji} {name}: なし\n"));
        }
        text.push_str(&format!("{ALL_ELEMENTS_EMOJI} 全属性可能: なし\n"));
        text
    } else {
        format!("{SIMPLE_JOIN_EMOJI} 参加: なし\n")
    }
}

/// 属性セレクトメニュー（複数選択可能）を作成する
///
/// # 引数
/// * `element_emojis` - カスタム属性絵文字
///
/// # 戻り値
/// CreateActionRow（セレクトメニュー）
pub fn create_element_select_menu(element_emojis: &ElementEmojis) -> CreateActionRow {
    use crate::types::ELEMENT_NAMES;

    let emojis_array = element_emojis.as_array();
    let mut options = Vec::new();

    // 属性1-6のオプション
    for (i, (emoji, name)) in emojis_array.iter().zip(ELEMENT_NAMES.iter()).enumerate() {
        let option = CreateSelectMenuOption::new(format!("{emoji} {name}"), format!("{}", i + 1));
        options.push(option);
    }

    let select_menu = CreateSelectMenu::new(
        "recruit_select_elements",
        CreateSelectMenuKind::String { options },
    )
    .placeholder("複数の属性を選択する")
    .min_values(1)
    .max_values(6); // 6属性

    CreateActionRow::SelectMenu(select_menu)
}

/// 募集用ボタンを作成する（ボタン版募集用）
///
/// # 引数
/// * `battle_style_name` - 攻略方法の名前（「6属性」かどうかで分岐）
/// * `element_emojis` - カスタム属性絵文字
///
/// # 戻り値
/// CreateActionRowのVec（Discord Message Componentsとして使用）
pub fn create_recruitment_buttons(
    battle_style_name: &str,
    element_emojis: &ElementEmojis,
) -> Vec<CreateActionRow> {
    use crate::types::{ALL_ELEMENTS_EMOJI, ELEMENT_NAMES};

    if battle_style_name == "6属性" {
        // 6属性の場合：属性1-6ボタン + 全属性可能ボタン
        let mut element_buttons = Vec::new();
        let emojis_array = element_emojis.as_array();
        for (i, (emoji, name)) in emojis_array.iter().zip(ELEMENT_NAMES.iter()).enumerate() {
            let button = CreateButton::new(format!("recruit_join_{}", i + 1))
                .label(format!("{emoji} {name}"))
                .style(ButtonStyle::Primary);
            element_buttons.push(button);
        }

        // 全属性可能ボタン
        let all_elements_button = CreateButton::new("recruit_join_0")
            .label(format!("{ALL_ELEMENTS_EMOJI} 全属性可能"))
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
            .label(format!("{SIMPLE_JOIN_EMOJI} 参加"))
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
    let mut components = create_recruitment_buttons(
        &recruitment_data.battle_style_name,
        &recruitment_data.element_emojis,
    );

    // 6属性の場合のみ、セレクトメニューを最後の行（全属性可能＋全て取り消し）の直前に挿入
    if recruitment_data.battle_style_name == "6属性" {
        // 最後の行（全属性可能＋全て取り消し）を取り出す
        let last_row = components.pop();

        // セレクトメニューを追加（選択時に即座に参加処理が実行される）
        let select_menu_row = create_element_select_menu(&recruitment_data.element_emojis);
        components.push(select_menu_row);

        // 最後の行を戻す
        if let Some(row) = last_row {
            components.push(row);
        }
    }

    // ボタン版用の初期参加者一覧を作成
    let initial_text = create_initial_participants_text_for_buttons(
        &recruitment_data.battle_style_name,
        &recruitment_data.element_emojis,
    );

    // ボタン版用のembedを作成（絵文字を修正済みのものを使用）
    let embed = CreateEmbed::new()
        .title("参加者一覧")
        .description(&initial_text)
        .footer(poise::serenity_prelude::CreateEmbedFooter::new(
            "参加者数: 0人",
        ))
        .color(0x0099ff);

    // deferした応答を完了させる形でボタン付きメッセージを送信
    let reply = CreateReply::default()
        .content(recruitment_data.message_content.clone())
        .embed(embed)
        .components(components);

    let message = ctx.send(reply).await?;
    Ok(message.message().await?.id.get())
}

/// 募集データを作成する（Repository直接アクセス版）
/// Facade層から呼び出すためのラッパー関数
pub async fn create_recruitment_data_with_repos<C>(
    db: &C,
    element_emojis: &ElementEmojis,
    params: RecruitmentParams<'_>,
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
        element_emojis,
        params,
    )
    .await
}
