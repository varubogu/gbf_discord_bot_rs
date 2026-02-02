//! 募集キャンセル処理のFacade層
//!
//! PoiseContext等のDiscordフレームワーク固有の型を扱い、
//! サービス層のビジネスロジックを呼び出す。

use crate::gateway::{DiscordMessageGateway, PoiseDiscordGateway};
use crate::infrastructure::database::container::RepositoryContainer;
use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::repository::battle_recruitments_repository::BattleRecruitmentsRepository;
use crate::repository::database::guild_settings_repository::SeaOrmGuildSettingsRepository;
use crate::repository::database::quest_repository::SeaOrmQuestRepository;
use crate::repository::database::recruitment_participants_repository::SeaOrmRecruitmentParticipantsRepository;
use crate::repository::{
    GuildSettingsRepository, QuestRepository, RecruitmentParticipantsRepository,
};
use crate::services::message::MessageService;
use crate::services::message::MessageTextId;
use crate::services::recruitment::cancel::{
    cancel_recruitment_by_message, check_can_cancel_recruitment, create_cancel_notification_text,
};
use crate::services::schedule::NotificationManagementService;
use crate::types;
use crate::types::discord::{DiscordChannelId, DiscordGuildId, DiscordMessageId, MessageContent};
use crate::types::{AppError, AppState, CanCancelResult, CancelOnDeleteResult, PoiseContext};
use poise::ReplyHandle;
use poise::serenity_prelude::{
    ChannelId, ComponentInteraction, ComponentInteractionCollector, Context,
    EditInteractionResponse, Message, MessageId,
};
use sea_orm::{ConnectionTrait, TransactionTrait};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, instrument, warn};

/// 募集をキャンセルできるか確認（公開関数）
#[instrument(
    level = "debug", 
    skip(ctx, message),
    fields(
        guild_id = %message.guild_id.map(|id| id.get()).unwrap_or(0),
        channel_id = %message.channel_id.get(),
        message_id = %message.id.get()
    )
)]
pub async fn can_cancel(
    ctx: PoiseContext<'_>,
    message: &Message,
) -> types::Result<CanCancelResult> {
    check_can_cancel_recruitment_internal(ctx, message).await
}

/// 募集キャンセルをユーザーに確認（公開関数）
#[instrument(
    level = "debug", 
    skip(ctx, message),
    fields(
        guild_id = %message.guild_id.map(|id| id.get()).unwrap_or(0),
        channel_id = %message.channel_id.get(),
        message_id = %message.id.get()
    )
)]
pub async fn confirm_cancel(ctx: PoiseContext<'_>, message: &Message) -> types::Result<()> {
    cancel_with_confirmation_internal(ctx, message).await
}

/// 募集をキャンセルできるか確認（内部関数）
#[instrument(
    level = "debug",
    skip(ctx, message),
    fields(
        guild_id = %message.guild_id.map(|id| id.get()).unwrap_or(0),
        channel_id = %message.channel_id.get(),
        message_id = %message.id.get()
    )
)]
async fn check_can_cancel_recruitment_internal(
    ctx: PoiseContext<'_>,
    message: &Message,
) -> types::Result<CanCancelResult> {
    info!("BattleRecruitmentFacade::cancel_recruitment - 募集をキャンセルします");

    let app_state = &ctx.data().app_state;
    let conn = app_state.guild_db();
    let txn = conn.begin().await?;

    // RLSポリシーのためにセッション変数を設定
    let guild_id = if let Some(guild_id) = ctx.guild_id() {
        guild_id.get()
    } else {
        warn!("guild_idを取得できませんでした");
        return Err(AppError::Business {
            message: "ギルド情報を取得できませんでした".to_string(),
        });
    };
    set_current_guild_id(&txn, guild_id as i64).await?;

    let channel_id = message.channel_id.get();
    let message_id = message.id.get();

    let result = async {
        // RepositoryContainerとRepositoryの取得
        let repos = RepositoryContainer::new();
        let battle_recruitment_repo = repos.battle_recruitment();

        // Gateway経由でDBの募集情報とDiscordメッセージの状況をチェック
        let gateway = PoiseDiscordGateway::new(Arc::clone(&ctx.serenity_context().http));
        let can_cancel_result = check_can_cancel_recruitment(
            &gateway,
            guild_id,
            channel_id,
            message_id,
            battle_recruitment_repo,
            &txn,
        )
        .await?;

        Ok::<CanCancelResult, crate::types::AppError>(can_cancel_result)
    }
    .await;

    match result {
        Ok(result) => {
            txn.commit().await?;
            info!(message_id = %message.id, "募集キャンセル可能性チェック完了");
            Ok(result)
        }
        Err(e) => {
            txn.rollback().await?;
            error!(error = %e, message_id = %message.id, "募集キャンセル可能性チェックエラー");
            Err(e)
        }
    }
}

/// 募集をキャンセルする（内部関数）
async fn cancel_recruitment_internal(
    ctx: PoiseContext<'_>,
    guild_id: u64,
    channel_id: u64,
    message_id: u64,
) -> types::Result<()> {
    info!("BattleRecruitmentFacade::cancel_recruitment - 募集をキャンセルします");

    let app_state = &ctx.data().app_state;
    let conn = app_state.guild_db();
    let txn = conn.begin().await?;

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id as i64).await?;

    let result = async {
        // RepositoryContainerとRepositoryの取得
        let repos = RepositoryContainer::new();
        let battle_recruitment_repo = repos.battle_recruitment();

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

        // 1. 募集メッセージを取得して内容を保存
        let channel_id_obj = ChannelId::from(channel_id);
        let original_message = channel_id_obj
            .message(&ctx.http(), MessageId::from(message_id))
            .await?;
        let original_content = original_message.content.clone();

        // 2. リアクションから参加者一覧を取得
        let participants = get_participants_from_reactions(ctx, channel_id, message_id).await?;

        // 3. ロケール情報とguild_id取得
        let locale = ctx.locale();
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
        let gateway = PoiseDiscordGateway::new(Arc::clone(&ctx.serenity_context().http));
        let message_content = MessageContent::text(&cancelled_content);
        gateway
            .edit_message(
                DiscordChannelId::new(channel_id),
                DiscordMessageId::new(message_id),
                message_content,
            )
            .await?;

        // 5. キャンセル通知メッセージを作成
        let cancel_notification = create_cancel_notification_text(
            &txn,
            message_service,
            guild_id_i64,
            locale,
            &participants,
        )
        .await?;

        // 5. キャンセル通知メッセージを送信
        let cancel_message_id =
            send_cancel_reply_message(ctx, channel_id, message_id, &cancel_notification).await?;

        // 6. DBから募集情報を取得し、キャンセル済み状態に更新
        let recruitment = cancel_recruitment_by_message(
            &txn,
            battle_recruitment_repo,
            guild_id,
            channel_id,
            message_id,
            DiscordMessageId::new(cancel_message_id.get()),
        )
        .await?;

        // 7. キャンセルした募集の関連通知を削除
        let notification_management_service = NotificationManagementService::new();
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

/// 募集キャンセル処理（確認付き）（内部関数）
async fn cancel_with_confirmation_internal(
    ctx: PoiseContext<'_>,
    message: &Message,
) -> types::Result<()> {
    // キャンセル可能かチェック
    let can_cancel_result = check_can_cancel_recruitment_internal(ctx, message).await?;

    // チェック以前に終了するパターン
    handle_cancel_check_result(ctx, can_cancel_result).await?;

    // 確認ボタンを表示してユーザーの応答を待機
    let reply = confirm_interaction(ctx).await?;
    let interaction = wait_for_user_confirmation(ctx, reply).await?;

    // ユーザーの選択に応じて処理を実行
    handle_user_choice(ctx, interaction, message).await
}

/// キャンセル可能性チェック結果の処理（内部関数）
async fn handle_cancel_check_result(
    ctx: PoiseContext<'_>,
    can_cancel_result: CanCancelResult,
) -> types::Result<()> {
    let (should_exit, exit_message) = is_exit(ctx, can_cancel_result).await;
    if should_exit && !exit_message.is_empty() {
        ctx.send(
            poise::CreateReply::default()
                .content(exit_message)
                .ephemeral(true),
        )
        .await?;
    }
    Ok(())
}

/// ユーザーの確認応答を待機（内部関数）
async fn wait_for_user_confirmation(
    ctx: PoiseContext<'_>,
    reply: ReplyHandle<'_>,
) -> types::Result<ComponentInteraction> {
    let component_interaction = ComponentInteractionCollector::new(ctx.serenity_context())
        .timeout(Duration::from_secs(30))
        .filter(move |mci| {
            mci.data.custom_id.starts_with("confirm_cancel")
                || mci.data.custom_id.starts_with("deny_cancel")
        })
        .await;

    match component_interaction {
        Some(interaction) => {
            interaction.defer(&ctx.http()).await?;
            Ok(interaction)
        }
        None => {
            reply
                .edit(
                    ctx,
                    poise::CreateReply::default()
                        .content("操作がタイムアウトしました。")
                        .components(vec![]),
                )
                .await?;
            Err(AppError::Business {
                message: "User confirmation timeout".to_string(),
            })
        }
    }
}

/// ユーザーの選択に応じた処理実行（内部関数）
async fn handle_user_choice(
    ctx: PoiseContext<'_>,
    interaction: ComponentInteraction,
    message: &Message,
) -> types::Result<()> {
    match interaction.data.custom_id.as_str() {
        "confirm_cancel" => handle_confirm_cancel(ctx, interaction, message).await,
        "deny_cancel" => handle_deny_cancel(ctx, interaction).await,
        _ => handle_unknown_choice(ctx, interaction).await,
    }
}

/// キャンセル確認時の処理（内部関数）
async fn handle_confirm_cancel(
    ctx: PoiseContext<'_>,
    interaction: ComponentInteraction,
    message: &Message,
) -> types::Result<()> {
    let guild_id = if let Some(guild_id) = ctx.guild_id() {
        guild_id.get()
    } else {
        warn!("guild_idを取得できませんでした");
        return Err(AppError::Business {
            message: "ギルド情報を取得できませんでした".to_string(),
        });
    };

    // 「キャンセル中...」に変更
    let app_state = &ctx.data().app_state;
    let conn = app_state.guild_db();
    let locale = ctx.locale();
    let guild_id_i64 = Some(guild_id as i64);
    let message_service = app_state.message_service();

    show_cancelling_message(
        ctx,
        &interaction,
        conn,
        message_service,
        guild_id_i64,
        locale,
    )
    .await?;

    let channel_id = message.channel_id.get();
    let message_id = message.id.get();

    match cancel_recruitment_internal(ctx, guild_id, channel_id, message_id).await {
        Ok(_) => {
            // キャンセル処理完了後、「キャンセル中...」メッセージを削除
            delete_cancelling_message(ctx, &interaction).await
        }
        Err(e) => {
            // エラーをユーザーに表示
            let error_msg = match &e {
                AppError::Business { message } => {
                    // ビジネスエラーの場合はメッセージのみ表示
                    message.clone()
                }
                _ => {
                    // その他のエラーの場合は詳細を含める
                    format!("キャンセル処理中にエラーが発生しました: {e}")
                }
            };
            interaction
                .edit_response(
                    &ctx.http(),
                    EditInteractionResponse::new()
                        .content(&error_msg)
                        .components(vec![]),
                )
                .await?;
            // エラーを伝播（処理失敗として扱う）
            Err(e)
        }
    }
}

/// キャンセル拒否時の処理（内部関数）
async fn handle_deny_cancel(
    ctx: PoiseContext<'_>,
    interaction: ComponentInteraction,
) -> types::Result<()> {
    delete_confirmation_message(ctx, &interaction).await
}

/// 不明な選択時の処理（内部関数）
async fn handle_unknown_choice(
    ctx: PoiseContext<'_>,
    interaction: ComponentInteraction,
) -> types::Result<()> {
    send_result_response(ctx, &interaction, "エラーが発生しました。".to_string()).await
}

/// 事前処理で終了するか判定（内部関数）
async fn is_exit(_ctx: PoiseContext<'_>, can_cancel_result: CanCancelResult) -> (bool, String) {
    match can_cancel_result {
        CanCancelResult::Success => (false, "".to_string()),
        CanCancelResult::AlreadyCancelled => {
            (true, "この募集は既にキャンセルされています。".to_string())
        }
        CanCancelResult::MessageDeleted => (true, "募集メッセージが削除されています。".to_string()),
        CanCancelResult::NotRecruitMessage => (
            true,
            "指定されたメッセージは募集メッセージではありません。".to_string(),
        ),
        CanCancelResult::NotFound => (true, "指定された募集が見つかりません。".to_string()),
        CanCancelResult::EventDatePassed => (
            true,
            "開催日時を過ぎているためキャンセルできません。".to_string(),
        ),
    }
}

// 未使用の関数を削除

/// 確認メッセージ表示（内部関数）
async fn confirm_interaction(ctx: PoiseContext<'_>) -> types::Result<ReplyHandle<'_>> {
    use poise::serenity_prelude::{ButtonStyle, CreateActionRow, CreateButton};

    // 確認メッセージとボタンを作成（直接poise型を構築）
    let confirm_button = CreateButton::new("confirm_cancel")
        .style(ButtonStyle::Danger)
        .label("はい");

    let cancel_button = CreateButton::new("deny_cancel")
        .style(ButtonStyle::Secondary)
        .label("いいえ");

    let action_row = CreateActionRow::Buttons(vec![confirm_button, cancel_button]);

    // 確認メッセージを送信
    let reply = ctx
        .send(
            poise::CreateReply::default()
                .content("この募集をキャンセルしますか？")
                .components(vec![action_row])
                .ephemeral(true),
        )
        .await?;

    Ok(reply)
}

/// コマンドのレスポンスを返す送信（内部関数）
async fn send_result_response(
    ctx: PoiseContext<'_>,
    interaction: &ComponentInteraction,
    content: String,
) -> types::Result<()> {
    // defer()済みのインタラクションにはedit_responseを使用
    interaction
        .edit_response(
            &ctx.serenity_context().http,
            EditInteractionResponse::new()
                .content(content)
                .components(vec![]),
        )
        .await?;
    Ok(())
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
/// # 戻り値
/// - `Ok(CancelOnDeleteResult)`: 処理結果
/// - `Err`: 処理中にエラーが発生
#[instrument(
    level = "debug",
    skip(ctx, app_state),
    fields(
        guild_id = %guild_id,
        channel_id = %channel_id,
        message_id = %message_id
    )
)]
pub async fn cancel_on_message_deleted(
    ctx: &Context,
    guild_id: u64,
    channel_id: u64,
    message_id: u64,
    app_state: &AppState,
) -> types::Result<CancelOnDeleteResult> {
    info!("cancel_on_message_deleted - メッセージ削除に伴う募集キャンセル処理を開始します");

    let conn = app_state.guild_db();
    let txn = conn.begin().await?;

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id as i64).await?;

    let result = async {
        // RepositoryContainerとRepositoryの取得
        let repos = RepositoryContainer::new();
        let battle_recruitment_repo = repos.battle_recruitment();

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
            battle_recruitment_repo,
            guild_id,
            channel_id,
            message_id,
            DiscordMessageId::new(message_id),
        )
        .await?;

        // 関連通知スケジュールを削除
        let notification_management_service = NotificationManagementService::new();
        notification_management_service
            .delete_recruitment_notifications(&txn, recruitment.id)
            .await?;

        // DBから参加者情報を取得
        let participants_repo = SeaOrmRecruitmentParticipantsRepository::new();
        let participant_user_ids = participants_repo
            .get_all_participant_user_ids(&txn, recruitment.id)
            .await?;

        // 参加者メンションを作成
        let participant_mentions: Vec<String> = participant_user_ids
            .into_iter()
            .map(|user_id| format!("<@{user_id}>"))
            .collect();

        // ギルド設定からロケールを取得
        let guild_settings_repo = SeaOrmGuildSettingsRepository::new();
        let locale = match guild_settings_repo
            .find_by_guild_id_with_txn(&txn, guild_id as i64)
            .await?
        {
            Some(settings) => settings.locale,
            None => "ja".to_string(),
        };

        // キャンセル通知メッセージを作成
        let message_service = MessageService::new();
        let notification_text = create_cancel_notification_text(
            &txn,
            &message_service,
            Some(guild_id as i64),
            Some(&locale),
            &participant_mentions,
        )
        .await?;

        // クエスト情報を取得して通知メッセージに追加
        let quest_repo = SeaOrmQuestRepository::new();
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
        let gateway = PoiseDiscordGateway::new(Arc::clone(&ctx.http));
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
            participants_count = participant_mentions.len(),
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

// ============================================================
// 以下はservice層から移動したDiscord操作関数（facade層で実装）
// ============================================================

/// リアクションから参加者一覧取得（facade層実装）
///
/// メッセージのリアクションを列挙し、各リアクションのユーザーを取得して
/// 参加者一覧を返す。ボットユーザーは除外される。
async fn get_participants_from_reactions(
    ctx: PoiseContext<'_>,
    channel_id: u64,
    message_id: u64,
) -> types::Result<Vec<String>> {
    info!(
        "リアクション参加者取得開始: channel_id={}, message_id={}",
        channel_id, message_id
    );

    let channel = ChannelId::from(channel_id);
    let message = channel
        .message(&ctx.http(), MessageId::from(message_id))
        .await?;

    let mut all_participants = Vec::new();

    for reaction in &message.reactions {
        // リアクションしたユーザーを取得
        match message
            .reaction_users(&ctx.http(), reaction.reaction_type.clone(), Some(100), None)
            .await
        {
            Ok(users) => {
                let user_mentions: Vec<String> = users
                    .iter()
                    .filter(|user| !user.bot) // ボットユーザーを除外
                    .map(|user| format!("<@{}>", user.id))
                    .collect();

                all_participants.extend(user_mentions);
            }
            Err(e) => {
                error!("リアクションユーザー取得エラー: {:?}", e);
                // エラーが発生しても他のリアクションの処理は続行
            }
        }
    }

    // 重複を除去
    all_participants.sort();
    all_participants.dedup();

    info!(
        "リアクション参加者取得完了: {} participants found",
        all_participants.len()
    );
    Ok(all_participants)
}

/// キャンセル返信送信（facade層実装）
async fn send_cancel_reply_message(
    ctx: PoiseContext<'_>,
    channel_id: u64,
    original_message_id: u64,
    content: &str,
) -> types::Result<MessageId> {
    info!(
        "キャンセル返信送信: channel_id={}, original_message_id={}",
        channel_id, original_message_id
    );

    let cancel_reply = ctx.say(content).await?;
    info!("キャンセル返信送信完了");

    Ok(cancel_reply.message().await?.id)
}

/// 確認メッセージを削除する（facade層実装）
async fn delete_confirmation_message(
    ctx: PoiseContext<'_>,
    interaction: &ComponentInteraction,
) -> types::Result<()> {
    info!("確認メッセージを削除します");

    // メッセージを削除
    interaction
        .message
        .delete(&ctx.serenity_context().http)
        .await?;

    info!("確認メッセージの削除完了");
    Ok(())
}

/// キャンセル中表示に変更する（facade層実装）
async fn show_cancelling_message<C>(
    ctx: PoiseContext<'_>,
    interaction: &ComponentInteraction,
    db: &C,
    message_service: &MessageService,
    guild_id: Option<i64>,
    locale: Option<&str>,
) -> types::Result<()>
where
    C: ConnectionTrait,
{
    info!("キャンセル中表示に変更します");

    // メッセージサービスからキャンセル中メッセージを取得
    let cancelling_message = message_service
        .get_message(
            db,
            MessageTextId::RecruitmentCommandCancellingProgress.as_str(),
            HashMap::new(),
            guild_id,
            locale,
        )
        .await
        .unwrap_or_else(|_| "キャンセル中...".to_string());

    interaction
        .edit_response(
            &ctx.serenity_context().http,
            EditInteractionResponse::new()
                .content(cancelling_message)
                .components(vec![]),
        )
        .await?;

    info!("キャンセル中表示への変更完了");
    Ok(())
}

/// キャンセル中メッセージを削除する（facade層実装）
async fn delete_cancelling_message(
    ctx: PoiseContext<'_>,
    interaction: &ComponentInteraction,
) -> types::Result<()> {
    info!("キャンセル中メッセージを削除します");

    interaction
        .message
        .delete(&ctx.serenity_context().http)
        .await?;

    info!("キャンセル中メッセージの削除完了");
    Ok(())
}
