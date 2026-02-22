//! 募集キャンセル処理のFacade層
//!
//! Gateway経由でDiscord APIを操作し、
//! サービス層のビジネスロジックを呼び出す。

use super::participant_mentions;
use crate::gateway::{DiscordMessageGateway, DiscordReactionGateway};
use crate::infrastructure::database::session::set_current_guild_id;
use crate::repository::{
    BattleRecruitmentsRepository, GuildSettingsRepository, QuestRepository,
    RecruitmentParticipantsRepository,
};
use crate::services::recruitment::cancel::{
    cancel_recruitment_by_message, check_can_cancel_recruitment, create_cancel_notification_text,
};
use crate::services::schedule::NotificationManagementService;
use crate::types;
use crate::types::discord::{DiscordChannelId, DiscordGuildId, DiscordMessageId, MessageContent};
use crate::types::{AppError, AppState, CanCancelResult, CancelOnDeleteResult};
use sea_orm::TransactionTrait;
use tracing::{error, info, instrument};

/// 募集をキャンセルできるか確認（公開関数）
///
/// # 引数
/// * `app_state` - アプリケーション状態
/// * `gateway` - Discord Gateway
/// * `guild_id` - ギルドID
/// * `channel_id` - チャンネルID
/// * `message_id` - メッセージID
#[instrument(
    level = "debug",
    skip(app_state, gateway),
    fields(
        guild_id = %guild_id.get(),
        channel_id = %channel_id.get(),
        message_id = %message_id.get()
    )
)]
pub async fn can_cancel<G>(
    app_state: &AppState,
    gateway: &G,
    guild_id: DiscordGuildId,
    channel_id: DiscordChannelId,
    message_id: DiscordMessageId,
) -> types::Result<CanCancelResult>
where
    G: DiscordMessageGateway + Sync,
{
    check_can_cancel_recruitment_internal(app_state, gateway, guild_id, channel_id, message_id)
        .await
}

/// 募集をキャンセルできるか確認（内部関数）
#[instrument(
    level = "debug",
    skip(app_state, gateway),
    fields(
        guild_id = %guild_id.get(),
        channel_id = %channel_id.get(),
        message_id = %message_id.get()
    )
)]
async fn check_can_cancel_recruitment_internal<G>(
    app_state: &AppState,
    gateway: &G,
    guild_id: DiscordGuildId,
    channel_id: DiscordChannelId,
    message_id: DiscordMessageId,
) -> types::Result<CanCancelResult>
where
    G: DiscordMessageGateway + Sync,
{
    info!("BattleRecruitmentFacade::cancel_recruitment - 募集をキャンセルします");

    let conn = app_state.guild_db();
    let txn = conn.begin().await?;

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id.get() as i64).await?;

    let result = async {
        // Repositoryの取得
        let battle_recruitment_repo = app_state.repositories.battle_recruitments;

        // Gateway経由でDBの募集情報とDiscordメッセージの状況をチェック
        let can_cancel_result = check_can_cancel_recruitment(
            gateway,
            guild_id.get(),
            channel_id.get(),
            message_id.get(),
            &battle_recruitment_repo,
            &txn,
        )
        .await?;

        Ok::<CanCancelResult, crate::types::AppError>(can_cancel_result)
    }
    .await;

    match result {
        Ok(result) => {
            txn.commit().await?;
            info!(message_id = %message_id.get(), "募集キャンセル可能性チェック完了");
            Ok(result)
        }
        Err(e) => {
            txn.rollback().await?;
            error!(error = %e, message_id = %message_id.get(), "募集キャンセル可能性チェックエラー");
            Err(e)
        }
    }
}

/// 募集をキャンセルする（公開関数）
///
/// キャンセル可能性チェック後、UI操作（確認ボタン表示など）はevents層で行い、
/// ユーザーが確認後にこの関数を呼び出す。
///
/// # 引数
/// * `app_state` - アプリケーション状態
/// * `gateway` - Discord Gateway
/// * `guild_id` - ギルドID
/// * `channel_id` - チャンネルID
/// * `message_id` - メッセージID
/// * `locale` - ロケール（オプション）
#[instrument(
    level = "debug",
    skip(app_state, gateway),
    fields(
        guild_id = %guild_id,
        channel_id = %channel_id,
        message_id = %message_id
    )
)]
pub async fn execute_cancel<G>(
    app_state: &AppState,
    gateway: &G,
    guild_id: u64,
    channel_id: u64,
    message_id: u64,
    locale: Option<&str>,
) -> types::Result<()>
where
    G: DiscordMessageGateway + DiscordReactionGateway + Sync,
{
    info!("BattleRecruitmentFacade::cancel_recruitment - 募集をキャンセルします");

    let conn = app_state.guild_db();
    let txn = conn.begin().await?;

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id as i64).await?;

    let result = async {
        // Repositoryの取得
        let battle_recruitment_repo = app_state.repositories.battle_recruitments;

        info!(
            "キャンセル処理開始: guild_id={}, channel_id={}, message_id={}",
            guild_id, channel_id, message_id
        );

        // 0. DBから募集情報を取得して開催日時をチェック
        // u64をドメイン型に変換
        let recruitment = battle_recruitment_repo
            .get_by_message_with_txn(
                &txn,
                DiscordGuildId::new(guild_id),
                DiscordChannelId::new(channel_id),
                DiscordMessageId::new(message_id),
            )
            .await?
            .ok_or_else(|| AppError::Business {
                message: "募集情報が見つかりません".to_string(),
            })?;

        // 開催日時を過ぎている場合はキャンセル不可
        let now = chrono::Utc::now();
        if recruitment.quest_start_at <= now {
            return Err(AppError::Business {
                message: "開催日時を過ぎているためキャンセルできません".to_string(),
            });
        }

        // 1. 募集メッセージを取得して内容を保存（Gatewayを使用）
        let channel_id_obj = DiscordChannelId::new(channel_id);
        let message_id_obj = DiscordMessageId::new(message_id);
        let original_message = gateway.get_message(channel_id_obj, message_id_obj).await?;
        let original_content = original_message.content.clone();

        // 2. DB参加者とリアクション参加者を合算して通知対象を作成
        let participants_repo = app_state.repositories.recruitment_participants;
        let participant_user_ids = participant_mentions::collect_notification_participant_user_ids(
            &participants_repo,
            gateway,
            &txn,
            recruitment.id,
            channel_id_obj,
            message_id_obj,
            &original_message,
        )
        .await?;

        // 3. ロケール情報とguild_id取得
        let guild_id_i64 = Some(guild_id as i64);
        let message_service = app_state.message_service();

        // 4. 募集メッセージを編集してキャンセル状態を明記（Gatewayを使用）
        let cancelled_content =
            crate::services::recruitment::cancel::create_cancelled_message_content(
                &txn,
                message_service,
                guild_id_i64,
                locale,
                &original_content,
            )
            .await?;
        let message_content = MessageContent::text(&cancelled_content);
        gateway
            .edit_message(channel_id_obj, message_id_obj, message_content)
            .await?;

        // 5. キャンセル通知メッセージを作成
        let cancel_notification = create_cancel_notification_text(
            &txn,
            message_service,
            guild_id_i64,
            locale,
            &participant_user_ids,
        )
        .await?;

        // 5. キャンセル通知メッセージを送信（Gatewayを使用）
        let cancel_message_id = gateway
            .send_reply(
                channel_id_obj,
                message_id_obj,
                MessageContent::text(&cancel_notification),
                Some("キャンセル通知".to_string()),
            )
            .await?;

        // 6. DBから募集情報を取得し、キャンセル済み状態に更新
        let recruitment = cancel_recruitment_by_message(
            &txn,
            &battle_recruitment_repo,
            guild_id,
            channel_id,
            message_id,
            cancel_message_id,
        )
        .await?;

        // 7. キャンセルした募集の関連通知を削除
        let notification_management_service = NotificationManagementService::new(
            app_state.repositories.notification,
            app_state.repositories.notification_rel_battle_recruitment,
            app_state.repositories.scheduled_task,
        );
        let deleted_count = notification_management_service
            .delete_recruitment_notifications(&txn, recruitment.id)
            .await?;

        info!(
            recruit_id = recruitment.id,
            deleted_notifications = deleted_count,
            "キャンセル処理完了"
        );

        Ok::<(), crate::types::AppError>(())
    }
    .await;

    match result {
        Ok(_) => {
            txn.commit().await?;
            info!(message_id = %message_id, "募集キャンセルが完了しました");
            Ok(())
        }
        Err(e) => {
            txn.rollback().await?;
            error!(error = %e, message_id = %message_id, "募集キャンセルエラー");
            Err(e)
        }
    }
}

/// メッセージ削除時の募集キャンセル処理（公開関数）
///
/// メッセージは既に削除されていますが、DBに保存された参加者情報を使用して
/// キャンセル通知を送信します。
///
/// # 実行内容
/// - DBを `is_canceled=true` に更新
/// - 関連する通知スケジュールを削除
/// - DBから参加者情報を取得
/// - 参加者へのメンション付きキャンセル通知を募集チャンネルに送信
///
/// # 引数
/// * `gateway` - Discord Gateway
/// * `guild_id` - ギルドID
/// * `channel_id` - チャンネルID
/// * `message_id` - メッセージID
/// * `app_state` - アプリケーション状態
///
/// # 戻り値
/// - `Ok(CancelOnDeleteResult)`: 処理結果
/// - `Err`: 処理中にエラーが発生
#[instrument(
    level = "debug",
    skip(gateway, app_state),
    fields(
        guild_id = %guild_id,
        channel_id = %channel_id,
        message_id = %message_id
    )
)]
pub async fn cancel_on_message_deleted<G>(
    gateway: &G,
    guild_id: u64,
    channel_id: u64,
    message_id: u64,
    app_state: &AppState,
) -> types::Result<CancelOnDeleteResult>
where
    G: DiscordMessageGateway + Sync,
{
    info!("cancel_on_message_deleted - メッセージ削除に伴う募集キャンセル処理を開始します");

    let conn = app_state.guild_db();
    let txn = conn.begin().await?;

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id as i64).await?;

    let result = async {
        // Repositoryの取得
        let battle_recruitment_repo = app_state.repositories.battle_recruitments;

        // 募集メッセージかどうか確認
        // u64をドメイン型に変換
        let recruitment_opt = battle_recruitment_repo
            .get_by_message_with_txn(
                &txn,
                DiscordGuildId::new(guild_id),
                DiscordChannelId::new(channel_id),
                DiscordMessageId::new(message_id),
            )
            .await?;

        let recruitment = match recruitment_opt {
            Some(r) => r,
            None => {
                // 募集メッセージではない
                info!(
                    message_id = %message_id,
                    "削除されたメッセージは募集メッセージではありませんでした"
                );
                return Ok::<CancelOnDeleteResult, AppError>(
                    CancelOnDeleteResult::NotRecruitmentMessage,
                );
            }
        };

        // 既にキャンセル済みの場合はスキップ
        if recruitment.is_canceled {
            info!(
                recruitment_id = recruitment.id,
                "既にキャンセル済みの募集のため処理をスキップします"
            );
            return Ok(CancelOnDeleteResult::AlreadyCancelled);
        }

        // 開催日時を過ぎている場合はキャンセル不要
        let now = chrono::Utc::now();
        if recruitment.quest_start_at <= now {
            info!(
                recruitment_id = recruitment.id,
                quest_start_at = %recruitment.quest_start_at,
                "開催日時を過ぎているためキャンセル対象外です"
            );
            return Ok(CancelOnDeleteResult::EventDatePassed);
        }

        info!(
            recruitment_id = recruitment.id,
            "募集メッセージの削除を検出、キャンセル処理を実行します"
        );

        // DBを is_canceled=true に更新
        // メッセージ削除時は元の募集メッセージIDをrecruit_end_message_idに設定
        cancel_recruitment_by_message(
            &txn,
            &battle_recruitment_repo,
            guild_id,
            channel_id,
            message_id,
            DiscordMessageId::new(message_id),
        )
        .await?;

        // 関連通知スケジュールを削除
        let notification_management_service = NotificationManagementService::new(
            app_state.repositories.notification,
            app_state.repositories.notification_rel_battle_recruitment,
            app_state.repositories.scheduled_task,
        );
        notification_management_service
            .delete_recruitment_notifications(&txn, recruitment.id)
            .await?;

        // DBから参加者情報を取得
        let participants_repo = app_state.repositories.recruitment_participants;
        let participant_user_ids = participants_repo
            .get_all_participant_user_ids_with_txn(&txn, recruitment.id)
            .await?;

        // ギルド設定からロケールを取得
        let guild_settings_repo = app_state.repositories.guild_settings;
        let locale = match guild_settings_repo
            .find_by_guild_id_with_txn(&txn, guild_id as i64)
            .await?
        {
            Some(settings) => settings.locale,
            None => "ja".to_string(),
        };

        // キャンセル通知メッセージを作成
        let message_service = app_state.message_service();
        let notification_text = create_cancel_notification_text(
            &txn,
            message_service,
            Some(guild_id as i64),
            Some(&locale),
            &participant_user_ids,
        )
        .await?;

        // クエスト情報を取得して通知メッセージに追加
        let quest_repo = app_state.repositories.quest;
        let final_notification_text = match quest_repo
            .get_by_target_id(&txn, recruitment.quest_id)
            .await?
        {
            Some(quest) => {
                let quest_start_at_jst = recruitment
                    .quest_start_at
                    .with_timezone(&chrono_tz::Asia::Tokyo);
                format!(
                    "【メッセージ削除によるキャンセル - {} / {}】\n{}",
                    quest.name,
                    quest_start_at_jst.format("%Y/%m/%d %H:%M"),
                    notification_text
                )
            }
            None => notification_text,
        };

        // 募集チャンネルに通知を送信（Gatewayを使用）
        let message_content = MessageContent::text(&final_notification_text);

        gateway
            .send_message(DiscordChannelId::new(channel_id), message_content)
            .await
            .map_err(|e| {
                error!(
                    error = %e,
                    recruitment_id = recruitment.id,
                    "キャンセル通知メッセージの送信に失敗しました"
                );
                AppError::Generic(e.to_string())
            })?;

        info!(
            recruitment_id = recruitment.id,
            participants_count = participant_user_ids.len(),
            "メッセージ削除に伴うキャンセル処理が完了しました"
        );

        Ok::<CancelOnDeleteResult, AppError>(CancelOnDeleteResult::Cancelled)
    }
    .await;

    match result {
        Ok(processed) => {
            txn.commit().await?;
            Ok(processed)
        }
        Err(e) => {
            txn.rollback().await?;
            error!(
                error = %e,
                guild_id = %guild_id,
                channel_id = %channel_id,
                message_id = %message_id,
                "メッセージ削除に伴うキャンセル処理でエラーが発生しました"
            );
            Err(e)
        }
    }
}
