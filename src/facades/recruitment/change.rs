use super::participant_mentions;
use crate::gateway::{DiscordMessageGateway, DiscordReactionGateway};
use crate::infrastructure::database::session::set_current_guild_id;
use crate::models::battle_recruitments::BattleRecruitments;
use crate::services::guild_environment_service::GuildEnvironmentService;
use crate::services::recruitment::new;
use crate::services::recruitment::quest_query_service::QuestQueryService;
use crate::services::recruitment::recruit_datetime_service::{
    RecruitDateTimeService, postpone_quest_departure,
};
use crate::services::recruitment::recruitment_participants_service::RecruitmentParticipantsService;
use crate::services::recruitment::recruitment_query_service::RecruitmentQueryService;
use crate::services::recruitment::recruitment_update_service::RecruitmentUpdateService;
use crate::services::recruitment::role_notification::RoleNotificationService;
use crate::services::schedule::{
    NotificationManagementService, RecruitmentMessageDeletionScheduleService,
};
use crate::services::timezone_service::TimezoneService;
use crate::types;
use crate::types::PostponeDepartureResult;
use crate::types::discord::{
    DiscordChannelId, DiscordGuildId, DiscordMessageId, EmbedContent, MessageContent, MessageData,
};
use chrono::{DateTime, Utc};
use sea_orm::TransactionTrait;

/// 出発日時の変更指定
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum EventDateChange {
    /// 変更しない
    #[default]
    Keep,
    /// 指定した日時へ変更する
    Set(DateTime<Utc>),
    /// 現在の出発日時から指定分だけ後ろへずらす
    PostponeMinutes(i64),
}

/// 募集変更内容
#[derive(Debug)]
pub struct RecruitmentChangeContent {
    /// クエスト名（変更する場合）
    pub quest: Option<String>,
    /// 開催日時の変更指定
    pub event_date: EventDateChange,
    /// 攻略方法ID（変更する場合）
    pub battle_style_id: Option<i32>,
}

/// 募集内容変更の結果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecruitmentChangeOutcome {
    /// 変更を適用した（適用後の出発日時）
    Applied { event_date: DateTime<Utc> },
    /// 既に出発時刻を過ぎているため後ろ倒しできなかった
    EventDatePassed,
}
use tracing::{debug, error, info, instrument, warn};

/// 募集変更用の出発日時をギルド設定のタイムゾーンで解析する。
///
/// DBアクセスとRLS設定はFacadeで完結させ、events層からService層への直接依存を防ぐ。
pub async fn parse_recruitment_event_date(
    app_state: &crate::types::AppState,
    guild_id: i64,
    input: &str,
) -> types::Result<DateTime<Utc>> {
    let txn = app_state.guild_db().begin().await?;
    set_current_guild_id(&txn, guild_id).await?;

    let result = RecruitDateTimeService::new(app_state.repositories.guild_settings)
        .parse_quest_departure_with_txn(&txn, guild_id, input)
        .await;

    match result {
        Ok(event_date) => {
            txn.commit().await?;
            Ok(event_date)
        }
        Err(error) => {
            txn.rollback().await?;
            Err(error)
        }
    }
}

/// 募集変更の権限を判定する
///
/// 募集主本人、または `gbf_bot_control` ロール保持者のみ変更できる。
/// `host_discord_user_id == 0` は旧データ（作成者不明）を表す。
fn ensure_can_change(
    recruitment: &BattleRecruitments,
    invoker_user_id: u64,
    has_bot_control: bool,
) -> types::Result<()> {
    let is_owner = recruitment.host_discord_user_id != 0
        && recruitment.host_discord_user_id == invoker_user_id;
    if !is_owner && !has_bot_control {
        return Err(types::AppError::Business {
            message:
                "この募集の変更は作成者本人または gbf_bot_control ロールを持つ管理者のみ可能です。"
                    .to_string(),
        });
    }
    Ok(())
}

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
        ensure_can_change(&recruitment, invoker_user_id, has_bot_control)?;

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

/// 募集の出発日時を指定分だけ遅らせる
///
/// 「募集内容変更」と同じ更新処理を、出発日時の相対シフト指定で呼び出す。
/// 現在の出発日時の読み取りと更新は単一トランザクション内で完結する。
///
/// # 引数
/// * `app_state` - アプリケーション状態
/// * `gateway` - Discord Gateway
/// * `guild_id` - ギルドID
/// * `message` - 対象メッセージ（ドメイン型）
/// * `delay_minutes` - 出発日時を遅らせる分数
/// * `invoker_user_id` - 操作を実行するユーザーのID
/// * `has_bot_control` - 実行者が gbf_bot_control ロールを保持しているか
#[instrument(level = "debug", skip(app_state, gateway, message))]
pub async fn postpone_recruitment_departure<G>(
    app_state: &crate::types::AppState,
    gateway: &G,
    guild_id: u64,
    message: &MessageData,
    delay_minutes: i64,
    invoker_user_id: u64,
    has_bot_control: bool,
) -> types::Result<RecruitmentChangeOutcome>
where
    G: DiscordMessageGateway + DiscordReactionGateway + crate::gateway::DiscordGuildGateway + Sync,
{
    change_recruitment_information_internal(
        app_state,
        gateway,
        guild_id,
        message,
        RecruitmentChangeContent {
            quest: None,
            event_date: EventDateChange::PostponeMinutes(delay_minutes),
            battle_style_id: None,
        },
        invoker_user_id,
        has_bot_control,
    )
    .await
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
) -> types::Result<RecruitmentChangeOutcome>
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
) -> types::Result<RecruitmentChangeOutcome>
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
        ensure_can_change(&existing_recruitment, invoker_user_id, has_bot_control)?;

        // 2. 出発日時を決定する
        // 相対シフト指定の場合は、同一トランザクションで読み取った既存の出発日時を基準にService層で算出する
        let (new_expiry_date, is_event_date_changed) = match content.event_date {
            EventDateChange::Keep => (existing_recruitment.quest_start_at, false),
            EventDateChange::Set(event_date) => (event_date, true),
            EventDateChange::PostponeMinutes(minutes) => {
                match postpone_quest_departure(
                    existing_recruitment.quest_start_at,
                    minutes,
                    Utc::now(),
                )? {
                    PostponeDepartureResult::Postponed(event_date) => (event_date, true),
                    PostponeDepartureResult::EventDatePassed => {
                        warn!(
                            recruitment_id = existing_recruitment.id,
                            quest_start_at = %existing_recruitment.quest_start_at,
                            "出発時刻を過ぎているため後ろ倒しできません"
                        );
                        // 何も変更せずに終了する
                        return Ok(RecruitmentChangeOutcome::EventDatePassed);
                    }
                }
            }
        };

        // 3. クエスト・攻略方法を決定（指定されていればそれを使用、未指定なら既存の値を使用）
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

        // 4. メッセージ表示用の募集データを作成
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

        // 5. 通知向け参加者を取得（DB + リアクションを合算）
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

        // 6. 更新後メッセージのEmbedを作成
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

        // 7. DBの募集情報を更新
        update_service
            .update_recruitment(
                &txn,
                existing_recruitment.id,
                new_quest_id,
                new_battle_style_id,
                new_expiry_date,
            )
            .await?;

        // 8. Discordのメッセージを更新（ドメインモデルを使用）
        let edit_content = MessageContent::new()
            .with_text(&recruitment_data.message_content)
            .with_embed(embed_for_update);

        gateway
            .edit_message(channel_id_obj, message_id_obj, edit_content)
            .await?;

        // 9. 変更通知メッセージを送信（ロールメンション + 参加者メンション）
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

        // 10. 出発日時が変更された場合、既存の通知を削除して新しい通知を作成
        if is_event_date_changed {
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

        Ok::<RecruitmentChangeOutcome, crate::types::AppError>(RecruitmentChangeOutcome::Applied {
            event_date: new_expiry_date,
        })
    }
    .await;

    match result {
        Ok(outcome) => {
            txn.commit().await?;
            info!("募集内容更新が完了しました: ");
            Ok(outcome)
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    /// テスト用の募集情報を生成する
    fn recruitment_for_test(host_discord_user_id: u64) -> BattleRecruitments {
        let now = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        BattleRecruitments {
            id: 1,
            guild_id: 100,
            channel_id: 200,
            message_id: 300,
            quest_id: 1,
            battle_style_id: 1,
            quest_start_at: Utc.with_ymd_and_hms(2026, 1, 1, 22, 0, 0).unwrap(),
            is_recruiting: true,
            is_canceled: false,
            recruit_end_message_id: None,
            full_notification_sent: false,
            host_discord_user_id,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn 募集主本人は変更できる() {
        // Arrange
        let recruitment = recruitment_for_test(42);

        // Act
        let result = ensure_can_change(&recruitment, 42, false);

        // Assert
        assert!(result.is_ok(), "募集主本人は変更できるべき");
    }

    #[test]
    fn bot制御ロール保持者は変更できる() {
        // Arrange
        let recruitment = recruitment_for_test(42);

        // Act
        let result = ensure_can_change(&recruitment, 99, true);

        // Assert
        assert!(
            result.is_ok(),
            "gbf_bot_controlロール保持者は変更できるべき"
        );
    }

    #[test]
    fn 第三者は変更できない() {
        // Arrange
        let recruitment = recruitment_for_test(42);

        // Act
        let result = ensure_can_change(&recruitment, 99, false);

        // Assert
        assert!(
            matches!(result, Err(types::AppError::Business { .. })),
            "第三者はBusinessエラーになるべき"
        );
    }

    #[test]
    fn 募集主不明の旧データは本人扱いにならない() {
        // Arrange: host_discord_user_id == 0 は作成者不明を表す
        let recruitment = recruitment_for_test(0);

        // Act
        let result = ensure_can_change(&recruitment, 0, false);

        // Assert
        assert!(
            matches!(result, Err(types::AppError::Business { .. })),
            "作成者不明の募集は本人扱いにならないべき"
        );
    }
}
