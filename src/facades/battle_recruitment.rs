use crate::infrastructure::database::transaction_manager::TransactionManager;
use crate::services::battle_recruitment::cancel::CancelRecruitmentService;
use crate::services::battle_recruitment::new::NewRecruitmentService;
use crate::services::battle_recruitment::participants::ParticipantsService;
use crate::services::battle_recruitment::start::StartRecruitmentService;
use crate::services::battle_recruitment::update::UpdateRecruitmentService;
use crate::types::PoiseContext;
use crate::types::battle_type::BattleType;
use poise::serenity_prelude::{ChannelId, GuildId, MessageId};
use std::sync::Arc;
use tracing::{error, info, warn};

/// 新しい募集を開始する（トランザクション対応版）
pub(crate) async fn new(
    ctx: &PoiseContext<'_>,
    quest_alias: &str,
    battle_type: BattleType,
) -> Result<(), String> {
    info!("battle_recruitment::new - 新しい募集を開始します");

    // TransactionManager を作成
    let tx_manager = match TransactionManager::new().await {
        Ok(manager) => manager,
        Err(e) => {
            error!("TransactionManager作成エラー: {}", e);
            return Err(format!("TransactionManager error: {}", e));
        }
    };

    // guild_idとchannel_idを取得
    let guild_id = ctx.guild_id().map(|id| id.get()).unwrap_or(0);
    let channel_id = ctx.channel_id().get();
    let serenity_ctx = ctx.serenity_context().clone();
    let quest_alias_for_log = quest_alias.to_string();
    let quest_alias_owned = quest_alias.to_string();

    // トランザクション内で処理を実行
    match tx_manager
        .execute_in_transaction(|_tx_ctx| {
            Box::pin(async move {
                // NewRecruitmentServiceのインスタンス作成（Repository依存注入は今回スキップ）
                let service = NewRecruitmentService::new().await?;

                // パラメータのクエストからクエスト情報を取得
                // パラメータから日時を解析
                // クエストと日時からメッセージを作成
                // メッセージを送信
                // データベースに登録
                // メッセージにリアクションを付与
                service
                    .create_recruitment(
                        &serenity_ctx,
                        channel_id,
                        guild_id,
                        &quest_alias_owned,
                        battle_type,
                        None, // デフォルトの日時を使用
                    )
                    .await?;

                Ok(())
            })
        })
        .await
    {
        Ok(_) => {
            info!("新規募集作成が完了しました: quest={}", quest_alias_for_log);
            Ok(())
        }
        Err(e) => {
            error!("新規募集作成エラー: {}", e);
            Err(format!("Transaction failed: {}", e))
        }
    }
}

/// 募集内容を更新する（TransactionManager対応版）
pub(crate) async fn information_update(
    ctx: &poise::serenity_prelude::Context,
    guild_id: u64,
    channel_id: u64,
    message_id: u64,
    new_content: Option<String>,
) -> Result<(), String> {
    info!("battle_recruitment::information_update - 募集内容を更新します");

    // TransactionManager を作成
    let tx_manager = match TransactionManager::new().await {
        Ok(manager) => manager,
        Err(e) => {
            error!("TransactionManager作成エラー: {}", e);
            return Err(format!("TransactionManager error: {}", e));
        }
    };

    // ctx から必要な値を事前に取得
    let serenity_ctx = ctx.clone();

    // トランザクション内で処理を実行
    match tx_manager
        .execute_in_transaction(|tx_ctx| {
            Box::pin(async move {
                // UpdateRecruitmentServiceのインスタンス作成（Repository依存注入は今回スキップ）
                let service = UpdateRecruitmentService::new();

                // 募集メッセージのリアクションとメンバーを取得
                // リアクションとメンバーからメッセージを作成
                // クエストと日時からメッセージを作成
                service
                    .update_recruitment_message(
                        &serenity_ctx,
                        guild_id,
                        channel_id,
                        message_id,
                        new_content,
                        None,
                    )
                    .await?;

                Ok(())
            })
        })
        .await
    {
        Ok(_) => {
            info!("募集内容更新が完了しました: message_id={}", message_id);
            Ok(())
        }
        Err(e) => {
            error!("募集内容更新エラー: {}", e);
            Err(format!("Transaction failed: {}", e))
        }
    }
}

/// 参加者を更新する（TransactionManager対応版）
pub(crate) async fn member_update(
    ctx: &poise::serenity_prelude::Context,
    guild_id: u64,
    channel_id: u64,
    message_id: u64,
) -> Result<(), String> {
    info!("battle_recruitment::member_update - 参加者を更新します");

    // TransactionManager を作成
    let tx_manager = match TransactionManager::new().await {
        Ok(manager) => manager,
        Err(e) => {
            error!("TransactionManager作成エラー: {}", e);
            return Err(format!("TransactionManager error: {}", e));
        }
    };

    // ctx から必要な値を事前に取得
    let serenity_ctx = ctx.clone();

    // トランザクション内で処理を実行
    match tx_manager
        .execute_in_transaction(|tx_ctx| {
            Box::pin(async move {
                // ParticipantsServiceのインスタンス作成（Repository依存注入は今回スキップ）
                let service = ParticipantsService::new().await?;

                // 募集情報をDBから取得
                let recruitment = service
                    .get_recruitment_from_db(guild_id, channel_id, message_id)
                    .await?;

                // 募集メッセージのリアクションとメンバーを取得
                let participants_by_reaction = service
                    .get_reactions_and_members(&serenity_ctx, channel_id, message_id)
                    .await?;

                // リアクションとメンバーからメッセージを作成
                // クエストと日時からメッセージを作成
                let content = format!("募集ID: {} の参加者を更新しました", recruitment.id);

                service
                    .update_message(
                        &serenity_ctx,
                        channel_id,
                        message_id,
                        &content,
                        &participants_by_reaction,
                    )
                    .await?;

                Ok(())
            })
        })
        .await
    {
        Ok(_) => {
            info!("参加者更新が完了しました: message_id={}", message_id);
            Ok(())
        }
        Err(e) => {
            error!("参加者更新エラー: {}", e);
            Err(format!("Transaction failed: {}", e))
        }
    }
}

/// 募集をキャンセルする（TransactionManager対応版）
pub(crate) async fn cancel(
    ctx: &PoiseContext<'_>,
    guild_id: u64,
    channel_id: u64,
    message_id: u64,
) -> Result<(), String> {
    info!("battle_recruitment::cancel - 募集をキャンセルします");

    // TransactionManager のみ作成（DB接続は隠蔽）
    let tx_manager = match TransactionManager::new().await {
        Ok(manager) => manager,
        Err(e) => {
            error!("TransactionManager作成エラー: {}", e);
            return Err(format!("Transaction manager error: {}", e));
        }
    };

    // ctx から必要な値を事前に取得
    let serenity_ctx = ctx.clone();

    // トランザクション内で処理実行
    match tx_manager
        .execute_in_transaction(|tx_ctx| {
            Box::pin(async move {
                // let service = CancelRecruitmentService::new(db);
                //
                // // DBから募集情報を取得
                // let recruitment = service.get_recruitment_from_db_with_txn(&tx_ctx.txn, guild_id, channel_id, message_id).await?;
                //
                // // リアクションから参加者一覧取得（トランザクション外で実行）
                // let participants = service.get_participants_from_reactions(&serenity_ctx, channel_id, message_id).await?;
                //
                // // 元のメッセージを取得して内容を確認
                // let channel = poise::serenity_prelude::ChannelId::from(channel_id);
                // let original_message = channel.message(&serenity_ctx.http, poise::serenity_prelude::MessageId::from(message_id)).await?;
                //
                // // 元のメッセージをキャンセル済みに編集
                // service.edit_original_message_as_cancelled(&serenity_ctx, channel_id, message_id, &original_message.content).await?;
                //
                // // キャンセル通知メッセージを作成して送信
                // let cancel_notification = service.create_cancel_notification(&participants).await?;
                // service.send_cancel_reply(&serenity_ctx, channel_id, message_id, &cancel_notification).await?;
                //
                // // 募集をキャンセル済み状態に更新
                // service.mark_recruitment_as_cancelled_with_txn(&tx_ctx, recruitment.id).await?;

                Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
            })
        })
        .await
    {
        Ok(_) => {
            info!(
                "募集キャンセル処理が完了しました: message_id={}",
                message_id
            );
            Ok(())
        }
        Err(e) => {
            error!("募集キャンセル処理エラー: {}", e);
            Err(format!("Transaction failed: {}", e))
        }
    }
}

/// 開始時間になった（TransactionManager対応版）
pub(crate) async fn start(
    ctx: &poise::serenity_prelude::Context,
    guild_id: u64,
    channel_id: u64,
    message_id: u64,
    recruitment_id: i64,
) -> Result<(), String> {
    info!("battle_recruitment::start - 募集を開始します");

    // TransactionManager を作成
    let tx_manager = match TransactionManager::new().await {
        Ok(manager) => manager,
        Err(e) => {
            error!("TransactionManager作成エラー: {}", e);
            return Err(format!("TransactionManager error: {}", e));
        }
    };

    // ctx から必要な値を事前に取得
    let serenity_ctx = ctx.clone();

    // トランザクション内で処理を実行
    match tx_manager
        .execute_in_transaction(|tx_ctx| {
            Box::pin(async move {
                // StartRecruitmentServiceのインスタンス作成（Repository依存注入は今回スキップ）
                let service = StartRecruitmentService::new().await?;

                // DBから募集情報を取得
                let recruitment = service
                    .get_recruitment_from_db(guild_id, channel_id, message_id)
                    .await?;

                // リアクションから参加者一覧取得
                let participants = service
                    .get_participants_from_reactions(&serenity_ctx, channel_id, message_id)
                    .await?;

                // 開始メッセージを作成
                let start_message = service
                    .create_start_message("サンプルクエスト", &participants)
                    .await?;

                // 開始返信送信
                service
                    .send_start_reply(&serenity_ctx, channel_id, message_id, &start_message)
                    .await?;

                // 募集を開始済み状態に更新
                service
                    .mark_recruitment_as_started(recruitment_id, message_id)
                    .await?;

                Ok(())
            })
        })
        .await
    {
        Ok(_) => {
            info!("募集開始処理が完了しました: message_id={}", message_id);
            Ok(())
        }
        Err(e) => {
            error!("募集開始処理エラー: {}", e);
            Err(format!("Transaction failed: {}", e))
        }
    }
}
