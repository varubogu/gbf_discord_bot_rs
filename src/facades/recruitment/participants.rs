use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::repository::database::battle_recruitments_repository::BattleRecruitmentsRepositoryImpl;
use crate::repository::database::recruitment_participants_repository::RecruitmentParticipantsRepositoryImpl;
use crate::services::recruitment::participants::ParticipantsService;
use crate::services::recruitment::quest_query_service::QuestQueryService;
use crate::services::recruitment::recruitment_participants_service::RecruitmentParticipantsService;
use crate::services::recruitment::recruitment_query_service::RecruitmentQueryService;
use crate::types;
use poise::serenity_prelude::Context;
use sea_orm::TransactionTrait;
use std::sync::Arc;
use tracing::{error, info, instrument, warn};

/// 参加者を更新する
///
/// # 引数
/// * `user_id` - リアクション追加/削除を行ったユーザーID（DB登録用、Noneの場合はDB登録なし）
/// * `reaction_emoji` - 追加/削除されたリアクション絵文字（DB登録用）
#[instrument(level = "debug", skip(ctx))]
pub async fn update_participants(
    ctx: &Context,
    guild_id: u64,
    channel_id: u64,
    message_id: u64,
    user_id: Option<u64>,
    reaction_emoji: Option<String>,
    db: &sea_orm::DatabaseConnection,
) -> types::Result<()> {
    info!("BattleRecruitmentFacade::update_participants - 参加者を更新します");
    let txn = db.begin().await?;

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id as i64).await?;

    let result = async {
        // Service層のインスタンスを作成
        let battle_recruitment_repo = Arc::new(BattleRecruitmentsRepositoryImpl::new());
        let participants_service = ParticipantsService::new(battle_recruitment_repo);
        let query_service = RecruitmentQueryService::new();
        let quest_query_service = QuestQueryService::new();

        // 募集情報の存在確認（キャンセル済み・期限切れチェック含む）
        let recruitment = match participants_service
            .update_participants_by_message(guild_id, channel_id, message_id, &txn)
            .await
        {
            Ok(recruitment) => recruitment,
            Err(types::AppError::Business { message }) => {
                // キャンセル済み・期限切れの場合は静かに処理を終了
                info!("募集更新スキップ: {}", message);
                return Ok(());
            }
            Err(e) => return Err(e),
        };

        // リアクション情報が提供されている場合、DBに登録/削除
        if let (Some(uid), Some(emoji)) = (user_id, reaction_emoji) {
            // battle_styleからリアクション絵文字リストを取得してelement_idを特定
            let battle_style = query_service
                .get_battle_style_by_id(&txn, recruitment.battle_style_id)
                .await?
                .ok_or_else(|| types::AppError::Business {
                    message: "攻略方法が見つかりませんでした".to_string(),
                })?;

            let element_id = if let Some(reactions_str) = &battle_style.reactions {
                let emojis: Vec<&str> = reactions_str.split(',').map(|s| s.trim()).collect();
                emojis
                    .iter()
                    .position(|&e| e == emoji)
                    .map(|pos| (pos + 1) as i32)
            } else {
                None
            };

            // DBに参加者を登録/削除（toggle）
            let participants_repo = RecruitmentParticipantsRepositoryImpl::new();
            let participants_svc =
                RecruitmentParticipantsService::<RecruitmentParticipantsRepositoryImpl>::new(
                    Arc::new(participants_repo),
                );

            match participants_svc
                .toggle_participation(&txn, recruitment.id, uid, element_id)
                .await
            {
                Ok(_) => info!(
                    "リアクション参加をDBに登録しました: recruitment_id={}, user_id={}, element_id={:?}",
                    recruitment.id, uid, element_id
                ),
                Err(e) => {
                    warn!("DB登録エラー（続行）: {}", e);
                    // エラーでも続行（embed更新は実行）
                }
            }
        }

        // メッセージのリアクションとユーザーを取得
        let participants_by_reaction = participants_service
            .get_reactions_and_members(ctx, channel_id, message_id)
            .await?;

        // 既存のメッセージ内容を取得
        let channel = poise::serenity_prelude::ChannelId::from(channel_id);
        let message = channel
            .message(&ctx.http, poise::serenity_prelude::MessageId::from(message_id))
            .await
            .map_err(crate::types::AppError::Discord)?;
        let message_content = message.content.clone();

        // 募集メッセージを編集して参加者一覧部分を反映
        participants_service
            .update_message(ctx, channel_id, message_id, &message_content, &participants_by_reaction)
            .await?;

        // 規定人数チェック
        let unique_participant_count = participants_service.count_unique_participants(&participants_by_reaction);
        info!("現在の参加者数: {}", unique_participant_count);

        // questの規定人数を取得
        if let Ok(quest) = quest_query_service.get_quest_by_id(db, recruitment.quest_id).await {
            let recruit_count = quest.recruit_count as usize;
            info!("規定人数: {}", recruit_count);

            // 規定人数に達した場合、通知を送信
            if unique_participant_count >= recruit_count {
                // 既に通知が送信されているかチェック
                let notification_sent = participants_service
                    .has_notification_been_sent(ctx, channel_id, message_id)
                    .await?;

                if !notification_sent {
                    info!("規定人数に到達しました。通知を送信します。");
                    let all_participants = participants_service.get_all_participants(&participants_by_reaction);

                    participants_service
                        .send_recruitment_full_notification(ctx, channel_id, message_id, all_participants)
                        .await?;
                } else {
                    info!("規定人数に達していますが、既に通知済みです。");
                }
            } else {
                info!("まだ規定人数に達していません。({}/{})", unique_participant_count, recruit_count);
            }
        } else {
            warn!("クエスト情報が見つかりませんでした: quest_id={}", recruitment.quest_id);
        }

        Ok::<(), crate::types::AppError>(())
    }
    .await;

    match result {
        Ok(_) => {
            txn.commit().await?;
            info!(message_id = %message_id, "参加者更新が完了しました");
            Ok(())
        }
        Err(e) => {
            txn.rollback().await?;
            error!(error = %e, message_id = %message_id, "参加者更新エラー");
            Err(e)
        }
    }
}
