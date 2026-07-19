use crate::gateway::DiscordMessageGateway;
use crate::services::guild_environment_service::{ElementEmojis, GuildEnvironmentService};
use crate::services::recruitment::recruitment_participants_service::RecruitmentParticipantsService;
use crate::services::recruitment::recruitment_query_service::RecruitmentQueryService;
use crate::types::discord::{DiscordChannelId, DiscordMessageId, EmbedContent, MessageContent};
use crate::types::{AppError, AppState, Result};
use tracing::{info, instrument};

/// 募集メッセージの参加者一覧を更新する
///
/// # 引数
/// * `gateway` - Discord Gateway
/// * `app_state` - アプリケーション状態
/// * `txn` - データベーストランザクション
/// * `recruitment` - 募集情報
/// * `message_id` - メッセージID
/// * `channel_id` - チャンネルID
#[instrument(level = "info", skip(gateway, app_state, txn))]
pub(super) async fn update_recruitment_message<G>(
    gateway: &G,
    app_state: &AppState,
    txn: &sea_orm::DatabaseTransaction,
    recruitment: &crate::models::battle_recruitments::BattleRecruitments,
    message_id: DiscordMessageId,
    channel_id: DiscordChannelId,
) -> Result<()>
where
    G: DiscordMessageGateway + crate::gateway::DiscordGuildGateway + Sync,
{
    info!("募集メッセージの参加者一覧を更新します");

    // 1. battle_styleの情報を取得（属性・絵文字の情報）
    let battle_style_repo = app_state.repositories.battle_style;
    let battle_recruitment_repo = app_state.repositories.battle_recruitments;
    let query_service = RecruitmentQueryService::new(battle_style_repo, battle_recruitment_repo);
    let battle_style = query_service
        .get_battle_style_by_id(txn, recruitment.battle_style_id)
        .await?
        .ok_or_else(|| AppError::Business {
            message: "攻略方法が見つかりませんでした".to_string(),
        })?;

    // 2. DBから参加者一覧を取得
    let participants_service =
        RecruitmentParticipantsService::new(app_state.repositories.recruitment_participants);
    let participants = participants_service
        .find_by_recruitment_id(txn, recruitment.id)
        .await?;

    // 2.5. 属性絵文字を取得（ギルド固有設定 or デフォルト値）
    let guild_env_repo = app_state.repositories.guild_environment;
    let guild_env_service = GuildEnvironmentService::new(guild_env_repo);
    let element_emojis = guild_env_service
        .get_element_emojis(txn, gateway, recruitment.guild_id as i64)
        .await?;

    // 3. 参加者一覧のテキストを作成
    let participants_text =
        create_participants_text(&battle_style.display_name, &participants, &element_emojis)
            .await?;

    // 3.5. ユニーク参加者数を計算（複数属性でも1人とカウント）
    use std::collections::HashSet;
    let unique_user_ids: HashSet<i64> = participants.iter().map(|p| p.user_id).collect();
    let participant_count = unique_user_ids.len();

    // 4. Gatewayを使ってメッセージを取得
    let message_data = gateway.get_message(channel_id, message_id).await?;

    // 既存のembedを取得（最初のembedを使用）
    let existing_embed = message_data.embeds.first().cloned();

    // 新しいembedを作成（既存の内容を保持しつつdescriptionとfooterを更新）
    let embed_content = if let Some(old_embed) = existing_embed {
        let mut embed = EmbedContent::new()
            .with_description(&participants_text)
            .with_footer(format!("参加者数: {participant_count}人"));
        if let Some(title) = &old_embed.title {
            embed = embed.with_title(title);
        }
        if let Some(color) = old_embed.color {
            embed = embed.with_color(color);
        }
        embed
    } else {
        // embedが存在しない場合は新規作成
        EmbedContent::new()
            .with_title("参加者一覧")
            .with_description(&participants_text)
            .with_footer(format!("参加者数: {participant_count}人"))
            .with_color(0x0099ff)
    };

    // Gatewayを使ってメッセージを更新
    let message_content = MessageContent::new().with_embed(embed_content);
    gateway
        .edit_message(channel_id, message_id, message_content)
        .await?;

    info!("募集メッセージの参加者一覧を更新しました");
    Ok(())
}

/// 参加者一覧のテキストを作成する
///
/// # 引数
/// * `battle_style_name` - 攻略方法の名前
/// * `participants` - 参加者一覧
/// * `element_emojis` - 属性絵文字
async fn create_participants_text(
    battle_style_name: &str,
    participants: &[crate::models::entities::worker::recruitment_participants::Model],
    element_emojis: &ElementEmojis,
) -> Result<String> {
    use std::collections::HashMap;

    // 属性IDごとに参加者をグループ化（Noneは0として扱う）
    let mut participants_by_element: HashMap<i32, Vec<u64>> = HashMap::new();
    for participant in participants {
        let element_id = participant.element_id.unwrap_or(0);
        participants_by_element
            .entry(element_id)
            .or_default()
            .push(participant.user_id as u64);
    }

    let mut text = String::new();

    // 6属性の場合
    if battle_style_name == "6属性" {
        use crate::types::{ALL_ELEMENTS_EMOJI, ELEMENT_NAMES};

        let emojis_array = element_emojis.as_array();
        for (idx, (emoji, name)) in emojis_array.iter().zip(ELEMENT_NAMES.iter()).enumerate() {
            let element_id = (idx + 1) as i32;
            if let Some(user_ids) = participants_by_element.get(&element_id) {
                let user_mentions: Vec<String> =
                    user_ids.iter().map(|&uid| format!("<@{uid}>")).collect();
                text.push_str(&format!(
                    "{} {}: {}\n",
                    emoji,
                    name,
                    user_mentions.join(" ")
                ));
            } else {
                text.push_str(&format!("{emoji} {name}: なし\n"));
            }
        }

        // 全属性可能（element_id = 0）
        if let Some(user_ids) = participants_by_element.get(&0) {
            let user_mentions: Vec<String> =
                user_ids.iter().map(|&uid| format!("<@{uid}>")).collect();
            text.push_str(&format!(
                "{} 全属性可能: {}\n",
                ALL_ELEMENTS_EMOJI,
                user_mentions.join(" ")
            ));
        } else {
            text.push_str(&format!("{ALL_ELEMENTS_EMOJI} 全属性可能: なし\n"));
        }
    } else {
        // シンプル参加の場合（element_id = null）
        use crate::types::SIMPLE_JOIN_EMOJI;

        if let Some(user_ids) = participants_by_element.get(&0) {
            let user_mentions: Vec<String> =
                user_ids.iter().map(|&uid| format!("<@{uid}>")).collect();
            text.push_str(&format!(
                "{} 参加: {}\n",
                SIMPLE_JOIN_EMOJI,
                user_mentions.join(" ")
            ));
        } else {
            text.push_str(&format!("{SIMPLE_JOIN_EMOJI} 参加: なし\n"));
        }
    }

    if text.is_empty() {
        Ok("現在参加者はいません。".to_string())
    } else {
        Ok(text)
    }
}
