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

#[derive(Debug)]
pub struct RecruitmentCanCancelParameters {
    pub guild_id: u64,
    pub channel_id: u64,
    pub message_id: u64,
}

#[derive(Debug)]
pub struct RecruitmentCancelContext {
    pub guild_id: u64,
    pub channel_id: u64,
    pub message_id: u64,
    pub cancel_message_id: u64,
    pub participants: Vec<String>,
    pub original_content: String,
    pub cancel_notification: String,
}

/// 募集をキャンセルできるか確認
#[instrument]
pub async fn confirm_cancel(ctx: PoiseContext<'_>, message: &Message) -> types::Result<()> {
    // 募集をキャンセルできるか確認
    can_cancel_recruitment(ctx, message)?;

    // 業務エラー、システムエラーならErrとして終了

    Ok(())
}

/// 募集をキャンセルできるか確認
#[instrument]
pub async fn execute_cancel(ctx: PoiseContext<'_>, message: &Message) -> types::Result<()> {
    Ok(())
}

/// 募集をキャンセルできるか確認
#[instrument]
pub async fn can_cancel_recruitment(ctx: PoiseContext<'_>, message: &Message) -> types::Result<()> {
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
            info!(message_id = %params.message_id, "募集キャンセル可能");
            Ok(result)
        }
        Err(e) => {
            txn.rollback().await?;
            error!(error = %e, message_id = %params.message_id, "募集キャンセルエラー");
            Err(e)
        }
    }
}

/// 募集をキャンセルする
#[instrument]
pub async fn cancel_recruitment(
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

pub async fn cancel___(ctx: PoiseContext<'_>, message: Message) -> types::Result<()> {
    let http = &ctx.http();

    let guild_id = ctx.guild_id().unwrap_or_default().get();
    let channel_id = message.channel_id.get();
    let message_id = message.id.get();

    // キャンセル可能かチェック
    let can_cancel_result = can_cancel_recruitment(ctx, guild_id, channel_id, message_id).await?;

    // チェック以前に終了するパターン
    match is_exit(ctx, can_cancel_result).await {
        Ok(_is_exit) => {
            if _is_exit {
                return Ok(());
            } else {
                // 処理続行
            }
        }
        Err(e) => {
            error!("{}", e);
            return Ok(());
        }
    }

    // キャンセル処理を続行するか確認するためのボタンを表示
    let reply = confirm_interaction(ctx).await?;

    // ボタンクリックを待機（30秒間）
    let component_interaction = ComponentInteractionCollector::new(ctx.serenity_context())
        .timeout(Duration::from_secs(30))
        .filter(move |mci| {
            mci.data.custom_id.starts_with("confirm_cancel")
                || mci.data.custom_id.starts_with("deny_cancel")
        })
        .await;

    // インタラクションが無効になっていたらその時点で終了
    let interaction = match component_interaction {
        Some(interaction) => interaction,
        None => {
            reply
                .edit(
                    ctx,
                    poise::CreateReply::default()
                        .content("操作がタイムアウトしました。")
                        .components(vec![]),
                )
                .await?;
            return Ok(());
        }
    };

    // インタラクションを待機して後で応答
    interaction.defer(http).await?;

    // キャンセル処理続行確認のボタンを押した
    match interaction.data.custom_id.as_str() {
        "confirm_cancel" => {
            let guild_id = ctx.guild_id().unwrap_or_default().get();
            let channel_id = message.channel_id.get();
            let message_id = message.id.get();

            match cancel_recruitment(ctx, guild_id, channel_id, message_id).await {
                Ok(_) => {
                    match send_result_response(
                        ctx,
                        interaction,
                        "募集がキャンセルされました。".to_string(),
                    )
                    .await
                    {
                        Ok(_) => Ok(()),
                        Err(_) => Ok(()),
                    }
                }
                Err(e) => match send_error_response(http, &interaction, e).await {
                    Ok(_) => Ok(()),
                    Err(_) => Ok(()),
                },
            }
        }
        "deny_cancel" => {
            // キャンセルをキャンセルされたら確認ボタン削除
            let reply_message = reply.into_message().await;
            match reply_message {
                Ok(msg) => msg.delete(ctx).await?,
                Err(_) => error!("a"),
            };
            match send_result_response(ctx, interaction, "キャンセルを取り消しました。".to_string())
                .await
            {
                Ok(_) => Ok(()),
                Err(_) => Ok(()),
            }
        }
        _ => {
            match send_result_response(ctx, interaction, "エラーが発生しました。".to_string()).await
            {
                Ok(_) => Ok(()),
                Err(_) => Ok(()),
            }
        }
    }
}

/// 事前処理で終了するか判定
pub async fn is_exit(ctx: PoiseContext<'_>, can_cancel_result: CanCancelResult) -> (bool, String) {
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

async fn send_cancel_response(ctx: PoiseContext<'_>, content: String) -> types::Result<()> {
    ctx.send(
        poise::CreateReply::default()
            .content("指定された募集が見つかりません。")
            .ephemeral(true),
    )
    .await?;
    Ok(())
}

/// 確認メッセージ表示
pub async fn confirm_interaction(ctx: PoiseContext<'_>) -> types::Result<ReplyHandle> {
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

/// コマンドのエラーレスポンス送信
pub async fn send_error_response(
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

/// コマンドのレスポンスを返す送信
pub async fn send_result_response(
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
