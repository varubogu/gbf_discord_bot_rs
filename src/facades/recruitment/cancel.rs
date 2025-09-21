use crate::infrastructure::database::container::RepositoryContainer;
use crate::services::recruitment::cancel::{
    cancel_recruitment_by_message, check_can_cancel_recruitment, create_cancel_notification_text,
    edit_original_message_as_cancelled, get_participants_from_reactions, send_cancel_reply_message,
};
use crate::types;
use crate::types::domain_interface_result::CanCancelResult;
use crate::types::{AppError, PoiseContext};
use poise::ReplyHandle;
use poise::serenity_prelude::{
    ButtonStyle, ChannelId, ComponentInteraction, ComponentInteractionCollector, CreateActionRow,
    CreateButton, CreateInteractionResponse, CreateInteractionResponseMessage, Http, Message,
    MessageId,
};
use sea_orm::TransactionTrait;
use std::time::Duration;
use tracing::{error, info, instrument};

// 未使用の構造体を削除（必要に応じて後で追加）

/// 募集をキャンセルできるか確認（公開関数）
#[instrument]
pub async fn can_cancel(ctx: PoiseContext<'_>, message: &Message) -> types::Result<CanCancelResult> {
    check_can_cancel_recruitment_internal(ctx, message).await
}

/// 募集キャンセルをユーザーに確認（公開関数）
#[instrument]
pub async fn confirm_cancel(ctx: PoiseContext<'_>, message: &Message) -> types::Result<()> {
    cancel_with_confirmation_internal(ctx, message).await
}

/// 募集をキャンセル実行（公開関数）
#[instrument]
pub async fn execute_cancel(ctx: PoiseContext<'_>, message: &Message) -> types::Result<()> {
    let guild_id = ctx.guild_id().unwrap_or_default().get();
    let channel_id = message.channel_id.get();
    let message_id = message.id.get();

    // キャンセル処理を実行
    cancel_recruitment_internal(ctx, guild_id, channel_id, message_id).await?;

    Ok(())
}

/// 募集をキャンセルできるか確認（内部関数）
#[instrument]
async fn check_can_cancel_recruitment_internal(ctx: PoiseContext<'_>, message: &Message) -> types::Result<CanCancelResult> {
    info!("BattleRecruitmentFacade::cancel_recruitment - 募集をキャンセルします");

    let app_state = &ctx.data().app_state;
    let conn = app_state.db();
    let txn = conn.begin().await?;

    let result = async {
        // RepositoryContainerとRepositoryの取得
        let repos = RepositoryContainer::new(conn);
        let battle_recruitment_repo = repos.battle_recruitment();

        // DBの募集情報とDiscordメッセージの状況をチェック
        let can_cancel_result =
            check_can_cancel_recruitment(ctx.serenity_context(), message, battle_recruitment_repo)
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
#[instrument]
async fn cancel_recruitment_internal(
    ctx: PoiseContext<'_>,
    guild_id: u64,
    channel_id: u64,
    message_id: u64,
) -> types::Result<()> {
    info!("BattleRecruitmentFacade::cancel_recruitment - 募集をキャンセルします");

    let app_state = &ctx.data().app_state;
    let conn = app_state.db();
    let txn = conn.begin().await?;

    let result = async {
        // RepositoryContainerとRepositoryの取得
        let repos = RepositoryContainer::new(conn);
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

        // 3. 募集メッセージを編集してキャンセル状態を明記
        edit_original_message_as_cancelled(ctx, channel_id, message_id, &original_content).await?;

        // 4. キャンセル通知メッセージを作成
        let cancel_notification = create_cancel_notification_text(&participants).await?;

        // 5. キャンセル通知メッセージを送信
        let cancel_message_id =
            send_cancel_reply_message(ctx, channel_id, message_id, &cancel_notification).await?;

        // 6. DBから募集情報を取得し、キャンセル済み状態に更新
        let _recruitment = cancel_recruitment_by_message(
            &txn,
            battle_recruitment_repo,
            guild_id,
            channel_id,
            message_id,
            cancel_message_id,
        )
        .await?;

        info!("キャンセル処理完了");

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
async fn cancel_with_confirmation_internal(ctx: PoiseContext<'_>, message: &Message) -> types::Result<()> {
    // キャンセル可能かチェック
    let can_cancel_result = check_can_cancel_recruitment_internal(ctx, &message).await?;

    // チェック以前に終了するパターン
    if let Err(e) = handle_cancel_check_result(ctx, can_cancel_result).await {
        return Err(e);
    }

    // 確認ボタンを表示してユーザーの応答を待機
    let reply = confirm_interaction(ctx).await?;
    let interaction = wait_for_user_confirmation(ctx, reply).await?;

    // ユーザーの選択に応じて処理を実行
    handle_user_choice(ctx, interaction, message).await
}

/// キャンセル可能性チェック結果の処理（内部関数）
async fn handle_cancel_check_result(ctx: PoiseContext<'_>, can_cancel_result: CanCancelResult) -> types::Result<()> {
    let (should_exit, exit_message) = is_exit(ctx, can_cancel_result).await;
    if should_exit {
        if !exit_message.is_empty() {
            ctx.send(
                poise::CreateReply::default()
                    .content(exit_message)
                    .ephemeral(true),
            )
            .await?;
        }
    }
    Ok(())
}

/// ユーザーの確認応答を待機（内部関数）
async fn wait_for_user_confirmation(ctx: PoiseContext<'_>, reply: ReplyHandle<'_>) -> types::Result<ComponentInteraction> {
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
    let guild_id = ctx.guild_id().unwrap_or_default().get();
    let channel_id = message.channel_id.get();
    let message_id = message.id.get();

    match cancel_recruitment_internal(ctx, guild_id, channel_id, message_id).await {
        Ok(_) => {
            send_result_response(ctx, &interaction, "募集がキャンセルされました。".to_string()).await
        }
        Err(e) => {
            send_error_response(&ctx.http(), &interaction, e).await?;
            Ok(())
        }
    }
}

/// キャンセル拒否時の処理（内部関数）
async fn handle_deny_cancel(ctx: PoiseContext<'_>, interaction: ComponentInteraction) -> types::Result<()> {
    send_result_response(ctx, &interaction, "キャンセルを取り消しました。".to_string()).await
}

/// 不明な選択時の処理（内部関数）
async fn handle_unknown_choice(ctx: PoiseContext<'_>, interaction: ComponentInteraction) -> types::Result<()> {
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

/// コマンドのエラーレスポンス送信（内部関数）
async fn send_error_response(
    http: &&Http,
    interaction: &ComponentInteraction,
    e: AppError,
) -> Result<(), AppError> {
    // エラーが発生した場合は、ボタンは残したままエラーメッセージを表示
    interaction
        .create_response(
            http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(format!("キャンセル処理中にエラーが発生しました: {}", e))
                    .ephemeral(true),
            ),
        )
        .await?;
    Ok(())
}

/// コマンドのレスポンスを返す送信（内部関数）
async fn send_result_response(
    ctx: PoiseContext<'_>,
    interaction: &ComponentInteraction,
    content: String,
) -> types::Result<()> {
    interaction
        .create_response(
            &ctx.serenity_context().http,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new()
                    .content(content)
                    .components(vec![]),
            ),
        )
        .await?;
    Ok(())
}
