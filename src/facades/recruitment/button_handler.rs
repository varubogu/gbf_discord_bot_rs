use crate::gateway::{DiscordMessageGateway, DiscordReactionGateway};
use crate::infrastructure::database::session::set_current_guild_id;
use crate::repository::{BattleRecruitmentsRepository, RecruitmentParticipantsRepository};
use crate::services::guild_environment_service::{ElementEmojis, GuildEnvironmentService};
use crate::services::recruitment::recruitment_participants_service::{
    ParticipationAction, RecruitmentParticipantsService,
};
use crate::services::recruitment::recruitment_query_service::RecruitmentQueryService;
use crate::types::constants::ELEMENT_NAMES;
use crate::types::discord::{
    DiscordChannelId, DiscordGuildId, DiscordMessageId, EmbedContent, MessageContent,
};
use crate::types::{AppError, AppState, RecruitmentComponentId, Result};
use sea_orm::TransactionTrait;
use tracing::{error, info, instrument};

/// ボタンハンドラーの処理結果
///
/// events層でインタラクション応答を行うための情報を含む
#[derive(Debug)]
pub struct ButtonHandlerResult {
    /// 応答メッセージ
    pub message: String,
}

impl ButtonHandlerResult {
    /// 新しい結果を作成
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// 属性セレクトメニューの選択を処理する（Facade層）
///
/// # 責務
/// - 選択された複数の属性で一括参加処理
/// - トランザクション境界の管理
/// - Service層の協調
///
/// # 引数
/// * `gateway` - Discord Gateway
/// * `app_state` - アプリケーション状態
/// * `guild_id` - ギルドID
/// * `channel_id` - チャンネルID
/// * `message_id` - メッセージID
/// * `user_id` - ユーザーID
/// * `element_ids` - 選択された属性IDのリスト
///
/// # 戻り値
/// 処理結果（応答メッセージを含む）
#[instrument(level = "info", skip(gateway, app_state))]
pub async fn handle_recruitment_select_menu<G>(
    gateway: &G,
    app_state: &AppState,
    guild_id: DiscordGuildId,
    channel_id: DiscordChannelId,
    message_id: DiscordMessageId,
    user_id: u64,
    element_ids: Vec<i32>,
) -> Result<ButtonHandlerResult>
where
    G: DiscordMessageGateway + DiscordReactionGateway + crate::gateway::DiscordGuildGateway + Sync,
{
    info!("属性セレクトメニュー処理開始");

    // DB接続とトランザクション開始
    let conn = app_state.guild_db();
    let txn = conn.begin().await?;

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id.get() as i64).await?;

    let result = async {
        // 1. メッセージIDから募集情報を取得
        let battle_style_repo = app_state.repositories.battle_style;
        let battle_recruitment_repo = app_state.repositories.battle_recruitments;
        let query_service =
            RecruitmentQueryService::new(battle_style_repo, battle_recruitment_repo);
        let recruitment = query_service
            .get_recruitment_by_message(&txn, guild_id.get(), channel_id.get(), message_id.get())
            .await?
            .ok_or_else(|| AppError::Business {
                message: "募集が見つかりませんでした".to_string(),
            })?;

        info!(recruitment_id = recruitment.id, "募集情報を取得しました");

        // 2. キャンセル済みチェック
        if recruitment.is_canceled {
            return Err(AppError::Business {
                message: "この募集はキャンセル済みです".to_string(),
            });
        }

        // 3. 期限切れチェック
        let now = chrono::Utc::now();
        if recruitment.quest_start_at < now {
            return Err(AppError::Business {
                message: "この募集は期限切れです".to_string(),
            });
        }

        // 4. Service層を使って複数属性の参加処理
        let participants_repo = app_state.repositories.recruitment_participants;
        let service = RecruitmentParticipantsService::new(participants_repo);

        let mut joined_elements = Vec::new();
        let mut left_elements = Vec::new();
        for element_id in &element_ids {
            let action = service
                .toggle_participation(
                    &txn,
                    recruitment.id,
                    user_id,
                    if *element_id == 0 {
                        None
                    } else {
                        Some(*element_id)
                    },
                )
                .await?;

            let element_name = if *element_id == 0 {
                "全属性可能".to_string()
            } else {
                ELEMENT_NAMES
                    .get((*element_id - 1) as usize)
                    .copied()
                    .unwrap_or("不明")
                    .to_string()
            };

            match action {
                ParticipationAction::Joined => joined_elements.push(element_name),
                ParticipationAction::Left => left_elements.push(element_name),
            }
        }

        // 参加と取り消しの両方のメッセージを生成
        let mut response_messages = Vec::new();

        if !joined_elements.is_empty() {
            response_messages.push(format!(
                "✅ {}属性で参加しました！",
                joined_elements.join(", ")
            ));
        }

        if !left_elements.is_empty() {
            response_messages.push(format!(
                "👋 {}属性の参加を取り消しました",
                left_elements.join(", ")
            ));
        }

        let response_message = if response_messages.is_empty() {
            "ℹ️ 変更はありませんでした".to_string()
        } else {
            response_messages.join("\n")
        };

        // 5. 参加者数を取得
        let participant_count = service
            .count_unique_participants(&txn, recruitment.id)
            .await?;

        info!(
            recruitment_id = recruitment.id,
            participant_count = participant_count,
            "参加者数を取得しました"
        );

        let participant_count_usize = participant_count.max(0) as usize;

        // 6. メッセージを更新して参加者一覧を反映
        update_recruitment_message(
            gateway,
            app_state,
            &txn,
            &recruitment,
            message_id,
            channel_id,
        )
        .await?;

        // 7. 規定人数到達の通知処理
        check_and_notify_recruitment_full(
            gateway,
            app_state,
            &txn,
            &recruitment,
            participant_count_usize,
            channel_id,
            message_id,
        )
        .await?;

        // 8. 応答メッセージを作成して返す
        let final_message =
            format!("{response_message}\n\n現在の参加者数: **{participant_count}人**");

        Ok(ButtonHandlerResult::new(final_message))
    }
    .await;

    match result {
        Ok(handler_result) => {
            txn.commit().await?;
            info!("属性セレクトメニュー処理が正常に完了しました");
            Ok(handler_result)
        }
        Err(e) => {
            txn.rollback().await?;
            error!(error = %e, "属性セレクトメニュー処理でエラーが発生しました");
            Err(e)
        }
    }
}

/// 募集ボタンのクリックを処理する（Facade層）
///
/// # 責務
/// - トランザクション境界の管理
/// - Service層の協調
///
/// # 引数
/// * `gateway` - Discord Gateway
/// * `app_state` - アプリケーション状態
/// * `guild_id` - ギルドID
/// * `channel_id` - チャンネルID
/// * `message_id` - メッセージID
/// * `user_id` - ユーザーID
/// * `custom_id` - コンポーネントのカスタムID
///
/// # 戻り値
/// 処理結果（応答メッセージを含む）
#[instrument(level = "info", skip(gateway, app_state))]
pub async fn handle_recruitment_button<G>(
    gateway: &G,
    app_state: &AppState,
    guild_id: DiscordGuildId,
    channel_id: DiscordChannelId,
    message_id: DiscordMessageId,
    user_id: u64,
    custom_id: &str,
) -> Result<ButtonHandlerResult>
where
    G: DiscordMessageGateway + DiscordReactionGateway + crate::gateway::DiscordGuildGateway + Sync,
{
    info!("募集ボタンクリック処理開始");

    // Custom IDをパース
    let component_id = RecruitmentComponentId::parse(custom_id)?;
    info!(component_id = ?component_id, "Custom IDをパースしました");

    // DB接続とトランザクション開始
    let conn = app_state.guild_db();
    let txn = conn.begin().await?;

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id.get() as i64).await?;

    let result = async {
        // 1. メッセージIDから募集情報を取得
        let battle_style_repo = app_state.repositories.battle_style;
        let battle_recruitment_repo = app_state.repositories.battle_recruitments;
        let query_service =
            RecruitmentQueryService::new(battle_style_repo, battle_recruitment_repo);
        let recruitment = query_service
            .get_recruitment_by_message(&txn, guild_id.get(), channel_id.get(), message_id.get())
            .await?
            .ok_or_else(|| AppError::Business {
                message: "募集が見つかりませんでした".to_string(),
            })?;

        info!(recruitment_id = recruitment.id, "募集情報を取得しました");

        // 2. キャンセル済みチェック
        if recruitment.is_canceled {
            return Err(AppError::Business {
                message: "この募集はキャンセル済みです".to_string(),
            });
        }

        // 3. 期限切れチェック
        let now = chrono::Utc::now();
        if recruitment.quest_start_at < now {
            return Err(AppError::Business {
                message: "この募集は期限切れです".to_string(),
            });
        }

        // 4. Service層を使って参加/退出処理
        let participants_repo = app_state.repositories.recruitment_participants;
        let service = RecruitmentParticipantsService::new(participants_repo);

        let response_message: String = match component_id {
            RecruitmentComponentId::Join => {
                // シンプル参加
                let action = service
                    .toggle_participation(&txn, recruitment.id, user_id, None)
                    .await?;
                match action {
                    ParticipationAction::Joined => "✅ 参加しました！".to_string(),
                    ParticipationAction::Left => "👋 参加を取り消しました".to_string(),
                }
            }
            RecruitmentComponentId::JoinElement(element_id) => {
                // 属性参加
                let element_name = ELEMENT_NAMES
                    .get((element_id - 1) as usize)
                    .copied()
                    .unwrap_or("不明");
                let action = service
                    .toggle_participation(&txn, recruitment.id, user_id, Some(element_id))
                    .await?;
                match action {
                    ParticipationAction::Joined => {
                        format!("✅ {element_name}属性で参加しました！")
                    }
                    ParticipationAction::Left => {
                        format!("👋 {element_name}属性の参加を取り消しました")
                    }
                }
            }
            RecruitmentComponentId::JoinAllElements => {
                // 全属性可能参加（element_idはNULL）
                let action = service
                    .toggle_participation(&txn, recruitment.id, user_id, None)
                    .await?;
                match action {
                    ParticipationAction::Joined => "✅ 全属性可能として参加しました！".to_string(),
                    ParticipationAction::Left => "👋 全属性可能参加を取り消しました".to_string(),
                }
            }
            RecruitmentComponentId::LeaveAll => {
                // すべて取り消し
                let count = service.leave_all(&txn, recruitment.id, user_id).await?;
                if count > 0 {
                    "👋 すべての参加を取り消しました".to_string()
                } else {
                    "ℹ️ 参加していませんでした".to_string()
                }
            }
            RecruitmentComponentId::SelectElements | RecruitmentComponentId::JoinSelected => {
                // セレクトメニュー自体のインタラクションはcomponent_interactionで処理されるためここには来ない
                // JoinSelectedも削除されたため、ここには来ない
                return Err(AppError::Business {
                    message: "予期しないコンポーネントIDです".to_string(),
                });
            }
        };

        // 5. 参加者数を取得
        let participant_count = service
            .count_unique_participants(&txn, recruitment.id)
            .await?;

        info!(
            recruitment_id = recruitment.id,
            participant_count = participant_count,
            "参加者数を取得しました"
        );

        let participant_count_usize = participant_count.max(0) as usize;

        // 6. メッセージを更新して参加者一覧を反映
        update_recruitment_message(
            gateway,
            app_state,
            &txn,
            &recruitment,
            message_id,
            channel_id,
        )
        .await?;

        // 7. 規定人数到達の通知処理
        check_and_notify_recruitment_full(
            gateway,
            app_state,
            &txn,
            &recruitment,
            participant_count_usize,
            channel_id,
            message_id,
        )
        .await?;

        // 8. 応答メッセージを作成して返す
        let final_message =
            format!("{response_message}\n\n現在の参加者数: **{participant_count}人**");

        Ok(ButtonHandlerResult::new(final_message))
    }
    .await;

    match result {
        Ok(handler_result) => {
            txn.commit().await?;
            info!("募集ボタンクリック処理が正常に完了しました");
            Ok(handler_result)
        }
        Err(e) => {
            txn.rollback().await?;
            error!(error = %e, "募集ボタンクリック処理でエラーが発生しました");
            Err(e)
        }
    }
}

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
async fn update_recruitment_message<G>(
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
    let participants_repo = app_state.repositories.recruitment_participants;
    let participants = participants_repo
        .find_by_recruitment_id_with_txn(txn, recruitment.id)
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

/// 規定人数到達の通知処理
///
/// # 引数
/// * `gateway` - Discord Gateway
/// * `app_state` - アプリケーション状態
/// * `txn` - データベーストランザクション
/// * `recruitment` - 募集情報
/// * `participant_count` - 現在の参加者数
/// * `channel_id` - チャンネルID
/// * `message_id` - メッセージID
#[instrument(level = "info", skip(gateway, app_state, txn))]
async fn check_and_notify_recruitment_full<G>(
    gateway: &G,
    app_state: &AppState,
    txn: &sea_orm::DatabaseTransaction,
    recruitment: &crate::models::battle_recruitments::BattleRecruitments,
    participant_count: usize,
    channel_id: DiscordChannelId,
    message_id: DiscordMessageId,
) -> Result<()>
where
    G: DiscordMessageGateway + Sync,
{
    info!("規定人数到達チェックを開始します");

    // クエスト情報を取得して規定人数を確認
    use crate::repository::QuestRepository;
    let quest_repository = app_state.repositories.quest;
    let quest = quest_repository
        .get_by_target_id(txn, recruitment.quest_id)
        .await?
        .ok_or_else(|| AppError::Business {
            message: "クエスト情報が見つかりませんでした".to_string(),
        })?;

    let required_count = quest.recruit_count as usize;
    let is_full = participant_count >= required_count;
    let notification_sent = recruitment.full_notification_sent;

    info!(
        participant_count = participant_count,
        required_count = required_count,
        is_full = is_full,
        notification_sent = notification_sent,
        "人数チェック結果"
    );

    // リポジトリを取得
    let recruitment_repo = app_state.repositories.battle_recruitments;

    match (notification_sent, is_full) {
        (false, false) => {
            // フラグ無し（未送信）で規定人数未満 → 何もしない
            info!("規定人数未達のため通知しません");
            Ok(())
        }
        (false, true) => {
            // フラグ無し（未送信）で規定人数以上 → フラグを立てて通知送信
            info!("規定人数に到達しました。通知を送信します");

            // 全参加者のメンションを取得
            let participants = get_all_participant_mentions(app_state, txn, recruitment.id).await?;

            // 通知メッセージを送信
            send_full_notification(gateway, channel_id, message_id, participants).await?;

            // フラグを立てる
            recruitment_repo
                .set_full_notification_sent_with_txn(txn, recruitment.id, true)
                .await?;

            info!("規定人数到達通知を送信しました");
            Ok(())
        }
        (true, false) => {
            // フラグあり（送信済）で規定人数未満 → フラグを下げて減少通知送信
            info!("参加者が規定人数を下回りました。通知を送信します");

            // 減少通知メッセージを送信
            send_decreased_notification(gateway, channel_id, message_id).await?;

            // フラグを下げる
            recruitment_repo
                .set_full_notification_sent_with_txn(txn, recruitment.id, false)
                .await?;

            info!("参加者減少通知を送信しました");
            Ok(())
        }
        (true, true) => {
            // フラグあり（送信済）で規定人数以上 → 何もしない
            info!("既に通知済みで規定人数以上のため何もしません");
            Ok(())
        }
    }
}

/// 全参加者のメンションを取得
///
/// # 引数
/// * `txn` - データベーストランザクション
/// * `recruitment_id` - 募集ID
async fn get_all_participant_mentions(
    app_state: &AppState,
    txn: &sea_orm::DatabaseTransaction,
    recruitment_id: i32,
) -> Result<Vec<String>> {
    use std::collections::HashSet;

    let participants_repo = app_state.repositories.recruitment_participants;
    let participants = participants_repo
        .find_by_recruitment_id_with_txn(txn, recruitment_id)
        .await?;

    // ユニークなユーザーIDを取得（重複排除）
    let unique_user_ids: HashSet<i64> = participants.iter().map(|p| p.user_id).collect();

    Ok(unique_user_ids
        .into_iter()
        .map(|user_id| format!("<@{user_id}>"))
        .collect())
}

/// 規定人数到達通知メッセージを送信
///
/// # 引数
/// * `gateway` - Discord Gateway
/// * `channel_id` - チャンネルID
/// * `message_id` - メッセージID
/// * `participants` - 参加者のメンション一覧
async fn send_full_notification<G>(
    gateway: &G,
    channel_id: DiscordChannelId,
    message_id: DiscordMessageId,
    participants: Vec<String>,
) -> Result<()>
where
    G: DiscordMessageGateway + Sync,
{
    let notification_message = format!("{}\n参加人数が集まりました。", participants.join(" "));

    // 返信形式で送信を試み、失敗時は文脈情報を付加して通常メッセージとして送信
    gateway
        .send_reply(
            channel_id,
            message_id,
            MessageContent::text(&notification_message),
            Some("規定人数到達通知".to_string()),
        )
        .await?;

    Ok(())
}

/// 参加者減少通知メッセージを送信
///
/// # 引数
/// * `gateway` - Discord Gateway
/// * `channel_id` - チャンネルID
/// * `message_id` - メッセージID
async fn send_decreased_notification<G>(
    gateway: &G,
    channel_id: DiscordChannelId,
    message_id: DiscordMessageId,
) -> Result<()>
where
    G: DiscordMessageGateway + Sync,
{
    let notification_message = "参加メンバーが規定人数を下回りました。";

    // 返信形式で送信を試み、失敗時は文脈情報を付加して通常メッセージとして送信
    gateway
        .send_reply(
            channel_id,
            message_id,
            MessageContent::text(notification_message),
            Some("参加者減少通知".to_string()),
        )
        .await?;

    Ok(())
}
