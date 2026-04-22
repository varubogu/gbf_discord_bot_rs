use super::participant_mentions;
use crate::gateway::{DiscordMessageGateway, DiscordReactionGateway};
use crate::infrastructure::database::session::set_current_guild_id;
use crate::services::guild_environment_service::GuildEnvironmentService;
use crate::services::recruitment::new;
use crate::services::recruitment::quest_query_service::QuestQueryService;
use crate::services::recruitment::recruitment_participants_service::RecruitmentParticipantsService;
use crate::services::recruitment::recruitment_query_service::RecruitmentQueryService;
use crate::services::recruitment::recruitment_update_service::RecruitmentUpdateService;
use crate::services::recruitment::role_notification::RoleNotificationService;
use crate::services::schedule::{
    NotificationManagementService, RecruitmentMessageDeletionScheduleService,
};
use crate::services::timezone_service::TimezoneService;
use crate::types;
use crate::types::discord::{
    DiscordChannelId, DiscordGuildId, DiscordMessageId, EmbedContent, MessageContent, MessageData,
};
use chrono::{DateTime, Utc};
use sea_orm::TransactionTrait;

/// 募集変更内容
#[derive(Debug)]
pub struct RecruitmentChangeContent {
    /// クエスト名（変更する場合）
    pub quest: Option<String>,
    /// 開催日時（変更する場合）
    pub event_date: Option<DateTime<Utc>>,
    /// 攻略方法ID（変更する場合）
    pub battle_style_id: Option<i32>,
}
use tracing::{debug, error, info, instrument};

/// 募集変更権限チェック（パネル表示前の早期チェック用）
///
/// # 引数
/// * `app_state` - アプリケーション状態
/// * `guild_id` - ギルドID
/// * `channel_id` - チャンネルID
/// * `message_id` - メッセージID
/// * `invoker_user_id` - 操作を実行するユーザーのID
/// * `has_bot_control` - 実行者が gbf_bot_control ロールを保持しているか
pub async fn check_can_change_recruitment(
    app_state: &crate::types::AppState,
    guild_id: u64,
    channel_id: u64,
    message_id: u64,
    invoker_user_id: u64,
    has_bot_control: bool,
) -> types::Result<()> {
    let txn = app_state.guild_db().begin().await?;
    set_current_guild_id(&txn, guild_id as i64).await?;

    let result = async {
        let query_service = RecruitmentQueryService::new(
            app_state.repositories.battle_style,
            app_state.repositories.battle_recruitments,
        );
        let recruitment = query_service
            .get_recruitment_by_message(&txn, guild_id, channel_id, message_id)
            .await?
            .ok_or_else(|| {
                types::AppError::NotFound("募集情報が見つかりませんでした".to_string())
            })?;

        // 権限チェック: 募集主本人または gbf_bot_control ロール保持者のみ変更可能
        // host_discord_user_id == 0 は旧データ（作成者不明）を表す
        let is_owner = recruitment.host_discord_user_id != 0
            && recruitment.host_discord_user_id == invoker_user_id;
        if !is_owner && !has_bot_control {
            return Err(types::AppError::Business {
                message:
                    "この募集の変更は作成者本人または gbf_bot_control ロールを持つ管理者のみ可能です。"
                        .to_string(),
            });
        }

        Ok::<(), types::AppError>(())
    }
    .await;

    match result {
        Ok(_) => {
            txn.commit().await?;
            Ok(())
        }
        Err(e) => {
            txn.rollback().await?;
            Err(e)
        }
    }
}

/// 募集内容を更新する
///
/// # 引数
/// * `app_state` - アプリケーション状態
/// * `gateway` - Discord Gateway
/// * `guild_id` - ギルドID
/// * `message` - 対象メッセージ（ドメイン型）
/// * `content` - 変更内容（クエスト名・開催日時・攻略方法ID）
/// * `invoker_user_id` - 操作を実行するユーザーのID
/// * `has_bot_control` - 実行者が gbf_bot_control ロールを保持しているか
#[instrument(level = "debug", skip(app_state, gateway, message))]
pub async fn change_recruitment_information<G>(
    app_state: &crate::types::AppState,
    gateway: &G,
    guild_id: DiscordGuildId,
    message: &MessageData,
    content: RecruitmentChangeContent,
    invoker_user_id: u64,
    has_bot_control: bool,
) -> types::Result<()>
where
    G: DiscordMessageGateway + DiscordReactionGateway + crate::gateway::DiscordGuildGateway + Sync,
{
    change_recruitment_information_internal(
        app_state,
        gateway,
        guild_id.get(),
        message,
        content,
        invoker_user_id,
        has_bot_control,
    )
    .await
}

/// 募集内容を更新する（内部実装 - PoiseContextに依存しない）
#[instrument(level = "debug", skip(app_state, gateway, message))]
pub async fn change_recruitment_information_internal<G>(
    app_state: &crate::types::AppState,
    gateway: &G,
    guild_id: u64,
    message: &MessageData,
    content: RecruitmentChangeContent,
    invoker_user_id: u64,
    has_bot_control: bool,
) -> types::Result<()>
where
    G: DiscordMessageGateway + DiscordReactionGateway + crate::gateway::DiscordGuildGateway + Sync,
{
    info!("BattleRecruitmentFacade::update_recruitment_information - 募集内容を更新します");

    let txn = app_state.guild_db().begin().await?;

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id as i64).await?;

    let result = async {
        // Serviceの取得
        let db = app_state.guild_db();
        let battle_style_repo = app_state.repositories.battle_style;
        let battle_recruitment_repo = app_state.repositories.battle_recruitments;
        let query_service =
            RecruitmentQueryService::new(battle_style_repo, battle_recruitment_repo);
        let quest_query_service = QuestQueryService::new(app_state.repositories.quest);
        let update_service = RecruitmentUpdateService::new(battle_recruitment_repo);
        let notification_service = NotificationManagementService::new(
            app_state.repositories.notification,
            app_state.repositories.notification_rel_battle_recruitment,
            app_state.repositories.scheduled_task,
        );
        let message_deletion_schedule_service = RecruitmentMessageDeletionScheduleService::new(
            app_state.repositories.guild_environment,
            app_state.repositories.environment,
            app_state.repositories.scheduled_task,
            app_state
                .repositories
                .scheduled_task_recruitment_message_deletion,
        );

        let channel_id = message.channel_id.get();
        let message_id = message.id.get();

        info!(
            guild_id = guild_id,
            channel_id = channel_id,
            message_id = message_id,
            "募集情報を検索します（guild_idはコンテキストから取得）"
        );

        // ドメイン型のIDオブジェクトを使用
        let channel_id_obj = DiscordChannelId::new(channel_id);
        let message_id_obj = DiscordMessageId::new(message_id);

        // 1. DBから既存の募集情報を取得
        let existing_recruitment = query_service
            .get_recruitment_by_message(&txn, guild_id, channel_id, message_id)
            .await?
            .ok_or_else(|| {
                error!(
                    guild_id = guild_id,
                    channel_id = channel_id,
                    message_id = message_id,
                    "募集情報がDBに見つかりませんでした"
                );
                types::AppError::NotFound("募集情報が見つかりませんでした".to_string())
            })?;

        // 権限チェック: 募集主本人または gbf_bot_control ロール保持者のみ変更可能
        // host_discord_user_id == 0 は旧データ（作成者不明）を表す
        let is_owner = existing_recruitment.host_discord_user_id != 0
            && existing_recruitment.host_discord_user_id == invoker_user_id;
        if !is_owner && !has_bot_control {
            return Err(types::AppError::Business {
                message:
                    "この募集の変更は作成者本人または gbf_bot_control ロールを持つ管理者のみ可能です。"
                        .to_string(),
            });
        }

        // 2. 更新する値を決定（指定されていればそれを使用、未指定なら既存の値を使用）
        let new_quest_id = if let Some(quest_name) = content.quest.as_deref() {
            // クエスト名が指定されている場合、新しいクエストを検索
            let quest = quest_query_service
                .search_and_get_quest_by_name(db, quest_name)
                .await?;

            quest.id
        } else {
            // クエスト名が指定されていない場合、既存の値を使用
            existing_recruitment.quest_id
        };

        let new_battle_style_id = if let Some(style_id) = content.battle_style_id {
            // 攻略方法が指定されている場合、それを使用
            style_id
        } else if content.quest.is_some() {
            // クエストが変更されている場合、新しいクエストのデフォルト攻略方法を使用
            let quest = quest_query_service
                .get_quest_by_id(db, new_quest_id)
                .await?;
            quest.default_battle_style_id
        } else {
            // どちらも指定されていない場合、既存の値を使用
            existing_recruitment.battle_style_id
        };

        let new_expiry_date = content.event_date.unwrap_or(existing_recruitment.quest_start_at);

        // タイムゾーンを取得
        let timezone_repo = app_state.repositories.guild_settings;
        let timezone_service = TimezoneService::new(timezone_repo);
        let timezone = timezone_service
            .get_guild_timezone_with_txn(&txn, guild_id as i64)
            .await?;

        // 属性絵文字を取得（ギルド固有設定 or デフォルト値）
        let guild_env_repo = app_state.repositories.guild_environment;
        let guild_env_service = GuildEnvironmentService::new(guild_env_repo);
        let element_emojis = guild_env_service
            .get_element_emojis(&txn, gateway, guild_id as i64)
            .await?;

        // 3. メッセージ表示用の募集データを作成
        let quest = quest_query_service
            .get_quest_by_id(db, new_quest_id)
            .await?;
        let quest_repo = app_state.repositories.quest;
        let battle_style_repo2 = app_state.repositories.battle_style;
        let message_service = app_state.message_service();
        let recruitment_data = new::create_recruitment_data(
            &txn,
            &quest_repo,
            &battle_style_repo2,
            &element_emojis,
            message_service,
            new::RecruitmentParams {
                quest_name_or_alias: &quest.name,
                battle_style_id: Some(new_battle_style_id),
                channel_id,
                guild_id,
                event_date: Some(new_expiry_date),
                timezone,
            },
        )
        .await?;

        // 4. 通知向け参加者を取得（DB + リアクションを合算）
        let participants_service =
            RecruitmentParticipantsService::new(app_state.repositories.recruitment_participants);
        let db_participant_user_ids = participants_service
            .get_all_participant_user_ids(&txn, existing_recruitment.id)
            .await?;
        let participant_user_ids = participant_mentions::collect_notification_participant_user_ids(
            db_participant_user_ids,
            gateway,
            channel_id_obj,
            message_id_obj,
            message,
        )
        .await?;
        let mentions = participant_mentions::to_mentions(&participant_user_ids).join(" ");

        // 5. 更新後メッセージのEmbedを作成
        // メッセージにコンポーネント（ボタン）があればv2、なければv1と判定
        let is_v2 = !message.components.is_empty();

        let embed_for_update = if is_v2 {
            // v2: DBから参加者を取得し、embed用のテキストも作成
            debug!("v2募集: DBから参加者を取得します");

            let participants = participants_service
                .find_by_recruitment_id(&txn, existing_recruitment.id)
                .await?;

            // ユニークなユーザーIDを取得（重複排除）
            let unique_user_ids: std::collections::HashSet<i64> =
                participants.iter().map(|p| p.user_id).collect();
            let participant_count = unique_user_ids.len();

            // 参加者一覧テキストを作成（embed用）
            let participants_text = create_participants_text_for_v2(
                &recruitment_data.battle_style_name,
                &participants,
                &element_emojis,
            );

            // ドメインモデルでEmbedを作成
            EmbedContent::new()
                .with_title("参加者一覧")
                .with_description(&participants_text)
                .with_footer(format!("参加者数: {participant_count}人"))
                .with_color(0x0099ff)
        } else {
            // v1は新規作成用のembedをそのまま使用（ドメインモデル）
            recruitment_data.embed_content.clone()
        };

        // 6. DBの募集情報を更新
        update_service
            .update_recruitment(
                &txn,
                existing_recruitment.id,
                new_quest_id,
                new_battle_style_id,
                new_expiry_date,
            )
            .await?;

        // 7. Discordのメッセージを更新（ドメインモデルを使用）
        let edit_content = MessageContent::new()
            .with_text(&recruitment_data.message_content)
            .with_embed(embed_for_update);

        gateway
            .edit_message(channel_id_obj, message_id_obj, edit_content)
            .await?;

        // 8. 変更通知メッセージを送信（ロールメンション + 参加者メンション）
        // ロールメンションを取得
        let all_roles_repo = app_state.repositories.all_recruitment_notification_roles;
        let quest_roles_repo = app_state.repositories.quest_recruitment_notification_roles;
        let role_service = RoleNotificationService::new(all_roles_repo, quest_roles_repo);
        let role_mentions = role_service
            .get_role_mentions(&txn, guild_id as i64, new_quest_id)
            .await?;

        debug!(
            role_mentions = %role_mentions,
            participant_mentions = %mentions,
            "変更通知メッセージを作成します"
        );

        // 変更通知メッセージを作成（ロールメンション + 参加者メンション）
        let update_notification = if role_mentions.is_empty() && mentions.is_empty() {
            "募集内容が更新されました。".to_string()
        } else {
            let mut parts = Vec::new();
            if !role_mentions.is_empty() {
                parts.push(role_mentions);
            }
            if !mentions.is_empty() {
                parts.push(mentions);
            }
            format!("{}\n募集内容が更新されました。", parts.join(" "))
        };

        // 返信形式で送信を試み、失敗時は文脈情報を付加して通常メッセージとして送信
        gateway
            .send_reply(
                channel_id_obj,
                message_id_obj,
                MessageContent::text(&update_notification),
                Some("募集内容変更通知".to_string()),
            )
            .await?;

        // 9. 出発日時が変更された場合、既存の通知を削除して新しい通知を作成
        if content.event_date.is_some() {
            // 既存の通知を削除
            notification_service
                .delete_recruitment_notifications(&txn, existing_recruitment.id)
                .await?;

            // 新しい通知を登録（5分前とちょうどの時刻）
            notification_service
                .create_recruitment_departure_notification(
                    &txn,
                    new_expiry_date,
                    guild_id as i64,
                    channel_id as i64,
                    existing_recruitment.id,
                )
                .await?;

            message_deletion_schedule_service
                .replace_for_recruitment(
                    &txn,
                    guild_id as i64,
                    channel_id as i64,
                    existing_recruitment.id,
                    new_expiry_date,
                )
                .await?;
        }

        info!("募集内容を更新しました");

        Ok::<(), crate::types::AppError>(())
    }
    .await;

    match result {
        Ok(_) => {
            txn.commit().await?;
            info!("募集内容更新が完了しました: ");
            Ok(())
        }
        Err(e) => {
            txn.rollback().await?;
            error!(error = %e, "募集内容更新エラー");
            Err(e)
        }
    }
}

/// v2募集用の参加者一覧テキストを作成する
///
/// # 引数
/// * `battle_style_name` - 攻略方法の名前
/// * `participants` - 参加者一覧（DBから取得）
/// * `element_emojis` - 属性絵文字
fn create_participants_text_for_v2(
    battle_style_name: &str,
    participants: &[crate::models::entities::worker::recruitment_participants::Model],
    element_emojis: &crate::services::guild_environment_service::ElementEmojis,
) -> String {
    use crate::types::{ALL_ELEMENTS_EMOJI, ELEMENT_NAMES, SIMPLE_JOIN_EMOJI};
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
        "現在参加者はいません。".to_string()
    } else {
        text
    }
}
