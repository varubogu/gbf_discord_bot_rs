use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::repository::database::guild_environment_repository::SeaOrmGuildEnvironmentRepository;
use crate::repository::database::guild_timezone_repository::GuildTimezoneRepository;
use crate::services::guild_environment_service::GuildEnvironmentService;
use crate::services::recruitment::new;
use crate::services::recruitment::quest_query_service::QuestQueryService;
use crate::services::recruitment::recruitment_query_service::RecruitmentQueryService;
use crate::services::recruitment::recruitment_update_service::RecruitmentUpdateService;
use crate::services::recruitment::role_notification::RoleNotificationService;
use crate::services::schedule::NotificationManagementService;
use crate::services::timezone_service::TimezoneService;
use crate::types;
use crate::types::PoiseContext;
use chrono::{DateTime, Duration, Utc};
use poise::serenity_prelude::Message;
use sea_orm::TransactionTrait;
use std::sync::Arc;
use tracing::{debug, error, info, instrument};

/// 募集内容を更新する（PoiseContext版）
#[instrument(level = "debug", skip(ctx, message))]
pub async fn change_recruitment_information(
    ctx: &PoiseContext<'_>,
    message: &Message,
    quest: Option<&str>,
    event_date: Option<DateTime<Utc>>,
    battle_style_id: Option<i32>,
) -> types::Result<()> {
    let app_state = &ctx.data().app_state;
    let guild_id = ctx.guild_id().map(|id| id.get()).unwrap_or(0);
    let http = ctx.http();

    change_recruitment_information_internal(
        app_state,
        http,
        guild_id,
        message,
        quest,
        event_date,
        battle_style_id,
    )
    .await
}

/// 募集内容を更新する（内部実装 - PoiseContextに依存しない）
#[instrument(level = "debug", skip(app_state, http, message))]
pub async fn change_recruitment_information_internal(
    app_state: &crate::types::AppState,
    http: &poise::serenity_prelude::Http,
    guild_id: u64,
    message: &Message,
    quest: Option<&str>,
    event_date: Option<DateTime<Utc>>,
    battle_style_id: Option<i32>,
) -> types::Result<()> {
    info!("BattleRecruitmentFacade::update_recruitment_information - 募集内容を更新します");

    let txn = app_state.guild_db().begin().await?;

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id as i64).await?;

    let result = async {
        // Serviceの取得
        let db = app_state.guild_db();
        let query_service = RecruitmentQueryService::new();
        let quest_query_service = QuestQueryService::new();
        let update_service = RecruitmentUpdateService::new();
        let notification_service = NotificationManagementService::new();

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

        // 2. 更新する値を決定（指定されていればそれを使用、未指定なら既存の値を使用）
        let new_quest_id = if let Some(quest_name) = quest {
            // クエスト名が指定されている場合、新しいクエストを検索
            let quest = quest_query_service
                .search_and_get_quest_by_name(db, quest_name)
                .await?;

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
            let quest = quest_query_service.get_quest_by_id(db, new_quest_id).await?;
            quest.default_battle_style_id
        } else {
            // どちらも指定されていない場合、既存の値を使用
            existing_recruitment.battle_style_id
        };

        let new_expiry_date = event_date.unwrap_or(existing_recruitment.quest_start_at);

        // タイムゾーンを取得
        let timezone_repo = Arc::new(GuildTimezoneRepository::new());
        let timezone_service = TimezoneService::new(timezone_repo);
        let timezone = timezone_service.get_guild_timezone(db, guild_id as i64).await?;

        // 属性絵文字を取得（ギルド固有設定 or デフォルト値）
        let guild_env_repo = Arc::new(SeaOrmGuildEnvironmentRepository::new());
        let guild_env_service = GuildEnvironmentService::new(guild_env_repo);
        let element_emojis = guild_env_service.get_element_emojis(db, http, guild_id as i64).await?;

        // 3. メッセージ表示用の募集データを作成
        let quest = quest_query_service.get_quest_by_id(db, new_quest_id).await?;
        let recruitment_data = new::create_recruitment_data_with_repos(
            db,
            &element_emojis,
            &quest.name,
            Some(new_battle_style_id),
            channel_id,
            guild_id,
            Some(new_expiry_date),
            timezone,
        )
        .await?;

        // 4. リアクションから参加者を取得
        let mut participant_ids = std::collections::HashSet::new();
        for reaction in &message.reactions {
            let users = channel_id_obj
                .reaction_users(http, message_id_obj, reaction.reaction_type.clone(), Some(100), None)
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
        update_service
            .update_recruitment(
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
            .edit_message(http, message_id_obj, edit_message)
            .await?;

        // 7. 変更通知メッセージを送信（ロールメンション + 参加者メンション）
        // ロールメンションを取得
        let role_service = RoleNotificationService::new();
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

        use poise::serenity_prelude::CreateMessage;
        let notification_message = CreateMessage::new()
            .content(update_notification)
            .reference_message((channel_id_obj, message_id_obj));

        channel_id_obj
            .send_message(http, notification_message)
            .await?;

        // 8. 出発日時が変更された場合、既存の通知を削除して新しい通知を作成
        if event_date.is_some() {
            // 既存の通知を削除
            notification_service
                .delete_recruitment_notifications(&txn, existing_recruitment.id)
                .await?;

            // 新しい通知を登録（出発5分前）
            let notify_time = new_expiry_date - Duration::minutes(5);
            notification_service
                .create_recruitment_departure_notification(
                    &txn,
                    notify_time,
                    guild_id as i64,
                    channel_id as i64,
                    existing_recruitment.id,
                )
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
