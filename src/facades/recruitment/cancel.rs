use crate::infrastructure::database::container::RepositoryContainer;
use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::services::recruitment::cancel::{
    cancel_recruitment_by_message, check_can_cancel_recruitment, create_cancel_notification_text,
    delete_cancelling_message, delete_confirmation_message, get_participants_from_reactions,
    send_cancel_reply_message, show_cancelling_message,
};
use crate::services::schedule::NotificationManagementService;
use crate::types;
use crate::types::domain_interface_result::CanCancelResult;
use crate::types::{AppError, PoiseContext};
use poise::ReplyHandle;
use poise::serenity_prelude::{
    ButtonStyle, ChannelId, ComponentInteraction, ComponentInteractionCollector, CreateActionRow,
    CreateButton, EditInteractionResponse, Message, MessageId,
};
use sea_orm::TransactionTrait;
use std::time::Duration;
use tracing::{error, info, instrument, warn};

// 未使用の構造体を削除（必要に応じて後で追加）

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

    let result = async {
        // RepositoryContainerとRepositoryの取得
        let repos = RepositoryContainer::new();
        let battle_recruitment_repo = repos.battle_recruitment();

        // DBの募集情報とDiscordメッセージの状況をチェック
        let can_cancel_result =
            check_can_cancel_recruitment(ctx, message, battle_recruitment_repo, &txn).await?;

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

        // 4. 募集メッセージを編集してキャンセル状態を明記
        let cancelled_content =
            crate::services::recruitment::cancel::create_cancelled_message_content(
                &txn,
                message_service,
                guild_id_i64,
                locale,
                &original_content,
            )
            .await?;
        let channel = ChannelId::from(channel_id);
        let edit_message = poise::serenity_prelude::EditMessage::new().content(cancelled_content);
        channel
            .edit_message(&ctx.http(), MessageId::from(message_id), edit_message)
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
            cancel_message_id,
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
            // エラーをユーザーに表示してから伝播
            let error_msg = format!("キャンセル処理中にエラーが発生しました: {e}");
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
    }
}

// 未使用の関数を削除

/// 確認メッセージ表示（内部関数）
async fn confirm_interaction(ctx: PoiseContext<'_>) -> types::Result<ReplyHandle> {
    // 確認メッセージとボタンを作成
    let confirm_button = CreateButton::new("confirm_cancel")
        .label("はい")
        .style(ButtonStyle::Danger);

    let cancel_button = CreateButton::new("deny_cancel")
        .label("いいえ")
        .style(ButtonStyle::Secondary);

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
