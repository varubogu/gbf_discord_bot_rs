use crate::infrastructure::database::container::RepositoryContainer;
use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::repository::database::battle_style_repository::SeaOrmBattleStyleRepository;
use crate::repository::database::quest_repository::SeaOrmQuestRepository;
use crate::repository::database::schedule::{
    NotificationRelBattleRecruitmentRepository, NotificationRepository,
};
use crate::repository::BattleRecruitmentsRepository;
use crate::repository::QuestRepository;
use crate::services::recruitment::new;
use crate::types;
use crate::types::PoiseContext;
use chrono::{DateTime, Duration, Utc};
use poise::serenity_prelude::Message;
use sea_orm::TransactionTrait;
use tracing::{error, info, instrument};

/// 募集内容を更新する（クロージャパターン）
#[instrument(level = "debug", skip(ctx, message))]
pub async fn change_recruitment_information(
    ctx: &PoiseContext<'_>,
    message: &Message,
    quest: Option<&str>,
    event_date: Option<DateTime<Utc>>,
    battle_style_id: Option<i32>,
) -> types::Result<()> {
    info!("BattleRecruitmentFacade::update_recruitment_information - 募集内容を更新します");

    let app_state = &ctx.data().app_state;
    let txn = app_state.guild_db().begin().await?;

    // コンテキストからguild_idを取得（メッセージオブジェクトのguild_idはNoneの可能性がある）
    let guild_id = ctx.guild_id().map(|id| id.get()).unwrap_or(0);

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id as i64).await?;

    let result = async {
        // RepositoryContainerとRepositoryの取得
        let db = app_state.guild_db();
        let repos = RepositoryContainer::new();
        let battle_recruitment_repo = repos.battle_recruitment();
        let quest_repository = SeaOrmQuestRepository::new();
        let battle_style_repository = SeaOrmBattleStyleRepository::new();
        let channel_id = message.channel_id.get();
        let message_id = message.id.get();

        info!(
            guild_id = guild_id,
            channel_id = channel_id,
            message_id = message_id,
            message_guild_id = ?message.guild_id,
            "募集情報を検索します（guild_idはコンテキストから取得）"
        );

        // Discord APIオブジェクトを作成
        use poise::serenity_prelude::{ChannelId, MessageId};
        let channel_id_obj = ChannelId::new(channel_id);
        let message_id_obj = MessageId::new(message_id);

        // 1. DBから既存の募集情報を取得
        let existing_recruitment = battle_recruitment_repo
            .get_by_message_with_txn(&txn, guild_id, channel_id, message_id)
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

        // 2. 更新する値を決定（指定されていればそれを使用、未指定なら既存の値を使用）
        let new_quest_id = if let Some(quest_name) = quest {
            // クエスト名が指定されている場合、新しいクエストを検索
            let search_results = quest_repository
                .search_by_name_or_alias(db, quest_name)
                .await?;

            let quest_search_result = search_results
                .first()
                .ok_or_else(|| types::AppError::NotFound(format!(
                    "クエスト '{}' が見つかりませんでした",
                    quest_name
                )))?;

            let quest = quest_repository
                .get_by_target_id(db, quest_search_result.quest_id)
                .await?
                .ok_or_else(|| types::AppError::NotFound(format!(
                    "クエストID {} の詳細情報が見つかりませんでした",
                    quest_search_result.quest_id
                )))?;

            quest.id
        } else {
            // クエスト名が指定されていない場合、既存の値を使用
            existing_recruitment.quest_id
        };

        let new_battle_style_id = if let Some(style_id) = battle_style_id {
            // 攻略方法が指定されている場合、それを使用
            style_id
        } else if quest.is_some() {
            // クエストが変更されている場合、新しいクエストのデフォルト攻略方法を使用
            let quest = quest_repository
                .get_by_target_id(db, new_quest_id)
                .await?
                .ok_or_else(|| types::AppError::NotFound(format!(
                    "クエストID {} が見つかりませんでした",
                    new_quest_id
                )))?;
            quest.default_battle_style_id
        } else {
            // どちらも指定されていない場合、既存の値を使用
            existing_recruitment.battle_style_id
        };

        let new_expiry_date = event_date.unwrap_or(existing_recruitment.quest_start_at);

        // 3. メッセージ表示用の募集データを作成
        let recruitment_data = new::create_recruitment_data(
            db,
            &quest_repository,
            &battle_style_repository,
            &quest_repository
                .get_by_target_id(db, new_quest_id)
                .await?
                .ok_or_else(|| types::AppError::NotFound(format!(
                    "クエストID {} が見つかりませんでした",
                    new_quest_id
                )))?
                .name,
            Some(new_battle_style_id),
            channel_id,
            guild_id,
            Some(new_expiry_date),
        )
        .await?;

        // 4. リアクションから参加者を取得
        let mut participant_ids = std::collections::HashSet::new();
        for reaction in &message.reactions {
            let users = channel_id_obj
                .reaction_users(&ctx.http(), message_id_obj, reaction.reaction_type.clone(), Some(100), None)
                .await?;

            for user in users {
                if !user.bot {
                    participant_ids.insert(user.id);
                }
            }
        }

        // 参加者メンションを作成
        let mut mentions = String::new();
        for user_id in participant_ids {
            mentions.push_str(&format!("<@{}> ", user_id));
        }

        // 5. DBの募集情報を更新
        battle_recruitment_repo
            .update_with_txn(
                &txn,
                existing_recruitment.id,
                new_quest_id,
                new_battle_style_id,
                new_expiry_date,
            )
            .await?;

        // 6. Discordのメッセージを更新
        use poise::serenity_prelude::EditMessage;

        let edit_message = EditMessage::new()
            .content(&recruitment_data.message_content)
            .embed(recruitment_data.embed.clone());

        channel_id_obj
            .edit_message(&ctx.http(), message_id_obj, edit_message)
            .await?;

        // 7. 変更通知メッセージを送信（参加者にメンション）
        let update_notification = if mentions.is_empty() {
            "募集内容が更新されました。".to_string()
        } else {
            format!("{}\n募集内容が更新されました。", mentions)
        };

        use poise::serenity_prelude::CreateMessage;
        let notification_message = CreateMessage::new()
            .content(update_notification)
            .reference_message((channel_id_obj, message_id_obj));

        channel_id_obj
            .send_message(&ctx.http(), notification_message)
            .await?;

        // 8. 出発日時が変更された場合、既存の通知を削除して新しい通知を作成
        if event_date.is_some() {
            let rel_repo = NotificationRelBattleRecruitmentRepository::new();
            let notification_repo = NotificationRepository::new();

            // 既存の通知リレーションを取得
            let old_relations = rel_repo.find_by_recruit_id_with_txn(&txn, existing_recruitment.id).await?;

            // 既存の通知を削除
            for relation in old_relations {
                rel_repo
                    .delete_by_notification_id_with_txn(&txn, relation.notification_id)
                    .await?;
                notification_repo
                    .delete_by_id_with_txn(&txn, relation.notification_id)
                    .await?;
            }

            // 新しい通知を登録（出発5分前）
            let notify_time = new_expiry_date - Duration::minutes(5);
            let notification = notification_repo
                .create_with_txn(
                    &txn,
                    notify_time,
                    guild_id as i64,
                    channel_id as i64,
                    "MSG00033".to_string(),
                )
                .await?;

            // 新しい通知リレーションを作成
            rel_repo
                .create_with_txn(&txn, existing_recruitment.id, notification.id)
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
