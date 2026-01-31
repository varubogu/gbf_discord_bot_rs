use chrono::{DateTime, Utc};
use std::collections::HashMap;
use tracing::info;

use crate::events::converters::{to_create_action_row, to_create_embed};
use crate::models::quests::Quest;
use crate::presenter::RecruitmentPresenter;
use crate::repository::QuestRepository;
use crate::repository::database::battle_style_repository::BattleStyleRepository;
use crate::services::guild_environment_service::ElementEmojis;
use crate::services::message::{MessageService, MessageTextId};
use crate::services::unified_datetime_parser::ParsedDismissalTime;
use crate::types;
use crate::types::PoiseContext;
use crate::types::discord::{
    ActionRowContent, DiscordChannelId, DiscordGuildId, DiscordMessageId, EmbedContent,
};
use poise::serenity_prelude::{CreateActionRow, ReactionType};
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
    pub embed_content: EmbedContent,
    pub reaction_emojis: Vec<String>,
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

    // reactionsをパース（絵文字文字列として取得）
    // 6属性の場合はelement_emojisから取得、それ以外はbattle_styleのreactionsをパース
    let reaction_emojis: Vec<String> = if battle_style.display_name == "6属性" {
        let emojis_array = element_emojis.as_array();
        emojis_array.iter().map(|emoji| emoji.to_string()).collect()
    } else {
        parse_reaction_emojis(battle_style.reactions.as_deref().unwrap_or("✅"))
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
        create_initial_participants_text(db, &reaction_emojis, Some(params.guild_id as i64))
            .await?;

    Ok(RecruitmentData {
        quest,
        battle_style_id: actual_battle_style_id,
        battle_style_name: battle_style.display_name,
        channel_id: params.channel_id,
        guild_id: params.guild_id,
        expiry_date,
        message_content,
        embed_content: EmbedContent::new()
            .with_title("参加者一覧")
            .with_description(&initial_participants_text)
            .with_color(0x0099ff),
        reaction_emojis,
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
        .embed(to_create_embed(&recruitment_data.embed_content));

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
    // u64をドメイン型に変換してRepositoryに渡す
    let recruitment = battle_recruitment_repo
        .create_with_txn(
            txn,
            crate::repository::CreateBattleRecruitmentParams {
                guild_id: DiscordGuildId::new(recruitment_data.guild_id),
                channel_id: DiscordChannelId::new(recruitment_data.channel_id),
                message_id: DiscordMessageId::new(message_id),
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

/// reactionsをパースする（カンマ区切りの絵文字文字列をStringのVecに変換）
fn parse_reaction_emojis(reactions_str: &str) -> Vec<String> {
    reactions_str
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

/// reactionsをパースする（カンマ区切りの絵文字文字列をReactionTypeのVecに変換）
/// 後方互換性のため残しているが、将来的には削除予定
#[allow(dead_code)]
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
    reaction_emojis: &[String],
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

    for emoji in reaction_emojis {
        text.push_str(&format!("{emoji} {no_participants}\n"));
    }

    if text.is_empty() {
        Ok("現在参加者はいません。".to_string())
    } else {
        Ok(text)
    }
}

/// ボタン版用の初期参加者一覧テキストを作成
/// Presenterへのラッパー関数
pub fn create_initial_participants_text_for_buttons(
    battle_style_name: &str,
    element_emojis: &ElementEmojis,
) -> String {
    RecruitmentPresenter::create_initial_participants_text(battle_style_name, element_emojis)
}

/// 募集用ボタンを作成する（ドメイン型版）
///
/// # 引数
/// * `battle_style_name` - 攻略方法の名前（「6属性」かどうかで分岐）
/// * `element_emojis` - カスタム属性絵文字
///
/// # 戻り値
/// ActionRowContentのVec（ドメインモデル）
pub fn create_recruitment_buttons(
    battle_style_name: &str,
    element_emojis: &ElementEmojis,
) -> Vec<ActionRowContent> {
    RecruitmentPresenter::create_recruitment_buttons(battle_style_name, element_emojis)
}

/// 属性セレクトメニュー（複数選択可能）を作成する（ドメイン型版）
///
/// # 引数
/// * `element_emojis` - カスタム属性絵文字
///
/// # 戻り値
/// ActionRowContent（ドメインモデル）
pub fn create_element_select_menu(element_emojis: &ElementEmojis) -> ActionRowContent {
    RecruitmentPresenter::create_element_select_menu(element_emojis)
}

/// 6属性募集用の全コンポーネント（ボタン + セレクトメニュー）を作成する（ドメイン型版）
///
/// # 引数
/// * `element_emojis` - カスタム属性絵文字
///
/// # 戻り値
/// ActionRowContentのVec（ドメインモデル）
pub fn create_six_element_full_components(element_emojis: &ElementEmojis) -> Vec<ActionRowContent> {
    RecruitmentPresenter::create_six_element_full_components(element_emojis)
}

/// Discord操作関数（ボタン付きメッセージ送信）
///
/// Presenterで生成したドメインモデルをConverterでpoise型に変換して送信する。
pub async fn send_recruitment_message_with_buttons(
    ctx: &PoiseContext<'_>,
    recruitment_data: &RecruitmentData,
) -> types::Result<u64> {
    use poise::CreateReply;

    // ボタンコンポーネントをPresenterから取得（ドメイン型）
    let components = if recruitment_data.battle_style_name == "6属性" {
        // 6属性の場合はセレクトメニュー付き
        RecruitmentPresenter::create_six_element_full_components(&recruitment_data.element_emojis)
    } else {
        // 通常の場合
        RecruitmentPresenter::create_recruitment_buttons(
            &recruitment_data.battle_style_name,
            &recruitment_data.element_emojis,
        )
    };

    // ドメイン型をpoise型に変換
    let poise_components: Vec<CreateActionRow> =
        components.iter().map(to_create_action_row).collect();

    // ボタン版用の初期参加者一覧を作成
    let initial_text = RecruitmentPresenter::create_initial_participants_text(
        &recruitment_data.battle_style_name,
        &recruitment_data.element_emojis,
    );

    // Presenterを使用してEmbedを生成
    let embed_content = RecruitmentPresenter::create_participants_embed(&initial_text, Some(0));

    // deferした応答を完了させる形でボタン付きメッセージを送信
    let reply = CreateReply::default()
        .content(recruitment_data.message_content.clone())
        .embed(to_create_embed(&embed_content))
        .components(poise_components);

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
