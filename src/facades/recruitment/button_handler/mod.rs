mod full_notification;
mod message_update;

use crate::gateway::{DiscordMessageGateway, DiscordReactionGateway};
use crate::infrastructure::database::session::set_current_guild_id;
use crate::services::recruitment::recruitment_participants_service::{
    ParticipationAction, RecruitmentParticipantsService,
};
use crate::services::recruitment::recruitment_query_service::RecruitmentQueryService;
use crate::types::constants::ELEMENT_NAMES;
use crate::types::discord::{DiscordChannelId, DiscordGuildId, DiscordMessageId};
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
        // 1〜3. 募集情報を取得し、キャンセル済み・期限切れでないことを検証
        let recruitment =
            load_active_recruitment(&txn, app_state, guild_id, channel_id, message_id).await?;

        // 4. 選択された複数属性の参加/取り消しをトグル
        let response_message =
            toggle_selected_elements(&txn, app_state, recruitment.id, user_id, &element_ids)
                .await?;

        // 5〜8. 参加者数取得→メッセージ更新→満員通知チェック→応答メッセージ組立
        finalize_participation_change(
            gateway,
            app_state,
            &txn,
            &recruitment,
            message_id,
            channel_id,
            response_message,
        )
        .await
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
        // 1〜3. 募集情報を取得し、キャンセル済み・期限切れでないことを検証
        let recruitment =
            load_active_recruitment(&txn, app_state, guild_id, channel_id, message_id).await?;

        // 4. コンポーネントIDに応じた参加/退出処理
        let response_message =
            toggle_by_component(&txn, app_state, recruitment.id, user_id, component_id).await?;

        // 5〜8. 参加者数取得→メッセージ更新→満員通知チェック→応答メッセージ組立
        finalize_participation_change(
            gateway,
            app_state,
            &txn,
            &recruitment,
            message_id,
            channel_id,
            response_message,
        )
        .await
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

/// メッセージIDから募集情報を取得し、キャンセル済み・期限切れでないことを検証する
///
/// `handle_recruitment_select_menu`と`handle_recruitment_button`の共通前処理
///
/// # 引数
/// * `txn` - データベーストランザクション
/// * `app_state` - アプリケーション状態
/// * `guild_id` - ギルドID
/// * `channel_id` - チャンネルID
/// * `message_id` - メッセージID
async fn load_active_recruitment(
    txn: &sea_orm::DatabaseTransaction,
    app_state: &AppState,
    guild_id: DiscordGuildId,
    channel_id: DiscordChannelId,
    message_id: DiscordMessageId,
) -> Result<crate::models::battle_recruitments::BattleRecruitments> {
    let battle_style_repo = app_state.repositories.battle_style;
    let battle_recruitment_repo = app_state.repositories.battle_recruitments;
    let query_service = RecruitmentQueryService::new(battle_style_repo, battle_recruitment_repo);
    let recruitment = query_service
        .get_recruitment_by_message(txn, guild_id.get(), channel_id.get(), message_id.get())
        .await?
        .ok_or_else(|| AppError::Business {
            message: "募集が見つかりませんでした".to_string(),
        })?;

    info!(recruitment_id = recruitment.id, "募集情報を取得しました");

    if recruitment.is_canceled {
        return Err(AppError::Business {
            message: "この募集はキャンセル済みです".to_string(),
        });
    }

    let now = chrono::Utc::now();
    if recruitment.quest_start_at < now {
        return Err(AppError::Business {
            message: "この募集は期限切れです".to_string(),
        });
    }

    Ok(recruitment)
}

/// 選択された複数属性の参加/取り消しをトグルし、応答メッセージを組み立てる
///
/// `handle_recruitment_select_menu`専用の参加処理本体
///
/// # 引数
/// * `txn` - データベーストランザクション
/// * `app_state` - アプリケーション状態
/// * `recruitment_id` - 募集ID
/// * `user_id` - ユーザーID
/// * `element_ids` - 選択された属性IDのリスト
async fn toggle_selected_elements(
    txn: &sea_orm::DatabaseTransaction,
    app_state: &AppState,
    recruitment_id: i32,
    user_id: u64,
    element_ids: &[i32],
) -> Result<String> {
    let participants_repo = app_state.repositories.recruitment_participants;
    let service = RecruitmentParticipantsService::new(participants_repo);

    let mut joined_elements = Vec::new();
    let mut left_elements = Vec::new();
    for element_id in element_ids {
        let action = service
            .toggle_participation(
                txn,
                recruitment_id,
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

    Ok(if response_messages.is_empty() {
        "ℹ️ 変更はありませんでした".to_string()
    } else {
        response_messages.join("\n")
    })
}

/// コンポーネントIDに応じた参加/退出処理を行い、応答メッセージを組み立てる
///
/// `handle_recruitment_button`専用の参加処理本体
///
/// # 引数
/// * `txn` - データベーストランザクション
/// * `app_state` - アプリケーション状態
/// * `recruitment_id` - 募集ID
/// * `user_id` - ユーザーID
/// * `component_id` - コンポーネントID
async fn toggle_by_component(
    txn: &sea_orm::DatabaseTransaction,
    app_state: &AppState,
    recruitment_id: i32,
    user_id: u64,
    component_id: RecruitmentComponentId,
) -> Result<String> {
    let participants_repo = app_state.repositories.recruitment_participants;
    let service = RecruitmentParticipantsService::new(participants_repo);

    let response_message = match component_id {
        RecruitmentComponentId::Join => {
            // シンプル参加
            let action = service
                .toggle_participation(txn, recruitment_id, user_id, None)
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
                .toggle_participation(txn, recruitment_id, user_id, Some(element_id))
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
                .toggle_participation(txn, recruitment_id, user_id, None)
                .await?;
            match action {
                ParticipationAction::Joined => "✅ 全属性可能として参加しました！".to_string(),
                ParticipationAction::Left => "👋 全属性可能参加を取り消しました".to_string(),
            }
        }
        RecruitmentComponentId::LeaveAll => {
            // すべて取り消し
            let count = service.leave_all(txn, recruitment_id, user_id).await?;
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

    Ok(response_message)
}

/// 参加者数取得→メッセージ更新→満員通知チェック→応答メッセージ組立を行う
///
/// `handle_recruitment_select_menu`と`handle_recruitment_button`の共通後処理
///
/// # 引数
/// * `gateway` - Discord Gateway
/// * `app_state` - アプリケーション状態
/// * `txn` - データベーストランザクション
/// * `recruitment` - 募集情報
/// * `message_id` - メッセージID
/// * `channel_id` - チャンネルID
/// * `response_message` - 参加/退出処理結果のメッセージ
#[allow(clippy::too_many_arguments)]
async fn finalize_participation_change<G>(
    gateway: &G,
    app_state: &AppState,
    txn: &sea_orm::DatabaseTransaction,
    recruitment: &crate::models::battle_recruitments::BattleRecruitments,
    message_id: DiscordMessageId,
    channel_id: DiscordChannelId,
    response_message: String,
) -> Result<ButtonHandlerResult>
where
    G: DiscordMessageGateway + DiscordReactionGateway + crate::gateway::DiscordGuildGateway + Sync,
{
    // 参加者数を取得
    let participants_repo = app_state.repositories.recruitment_participants;
    let service = RecruitmentParticipantsService::new(participants_repo);
    let participant_count = service
        .count_unique_participants(txn, recruitment.id)
        .await?;

    info!(
        recruitment_id = recruitment.id,
        participant_count = participant_count,
        "参加者数を取得しました"
    );

    let participant_count_usize = participant_count.max(0) as usize;

    // メッセージを更新して参加者一覧を反映
    message_update::update_recruitment_message(
        gateway,
        app_state,
        txn,
        recruitment,
        message_id,
        channel_id,
    )
    .await?;

    // 規定人数到達の通知処理
    full_notification::check_and_notify_recruitment_full(
        gateway,
        app_state,
        txn,
        recruitment,
        participant_count_usize,
        channel_id,
        message_id,
    )
    .await?;

    // 応答メッセージを作成して返す
    let final_message = format!("{response_message}\n\n現在の参加者数: **{participant_count}人**");

    Ok(ButtonHandlerResult::new(final_message))
}
