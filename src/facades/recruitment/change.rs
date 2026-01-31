use crate::events::converters::to_edit_message;
use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::repository::database::guild_environment_repository::SeaOrmGuildEnvironmentRepository;
use crate::repository::database::guild_settings_repository::SeaOrmGuildSettingsRepository;
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
use crate::types::discord::{EmbedContent, MessageContent};
use crate::utils::discord_helper::send_message_with_optional_reply;
use chrono::{DateTime, Utc};
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
            let quest = quest_query_service
                .get_quest_by_id(db, new_quest_id)
                .await?;
            quest.default_battle_style_id
        } else {
            // どちらも指定されていない場合、既存の値を使用
            existing_recruitment.battle_style_id
        };

        let new_expiry_date = event_date.unwrap_or(existing_recruitment.quest_start_at);

        // タイムゾーンを取得
        let timezone_repo = Arc::new(SeaOrmGuildSettingsRepository::new());
        let timezone_service = TimezoneService::new(timezone_repo);
        let timezone = timezone_service
            .get_guild_timezone(db, guild_id as i64)
            .await?;

        // 属性絵文字を取得（ギルド固有設定 or デフォルト値）
        let guild_env_repo = Arc::new(SeaOrmGuildEnvironmentRepository::new());
        let guild_env_service = GuildEnvironmentService::new(guild_env_repo);
        let element_emojis = guild_env_service
            .get_element_emojis(db, http, guild_id as i64)
            .await?;

        // 3. メッセージ表示用の募集データを作成
        let quest = quest_query_service
            .get_quest_by_id(db, new_quest_id)
            .await?;
        let recruitment_data = new::create_recruitment_data_with_repos(
            db,
            &element_emojis,
            new::RecruitmentParams {
                quest_name_or_alias: &quest.name,
                battle_style_id: Some(new_battle_style_id),
                channel_id,
                guild_id,
                event_date: Some(new_expiry_date),
                timezone,
            },
        )
        .await?;

        // 4. 参加者を取得（v2はDBから、v1はリアクションから）
        // メッセージにコンポーネント（ボタン）があればv2、なければv1と判定
        let is_v2 = !message.components.is_empty();

        // v2用: DBから参加者一覧を取得
        use crate::models::entities::worker::recruitment_participants::{
            Column as ParticipantColumn, Entity as RecruitmentParticipantEntity,
        };
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

        let (mentions, embed_for_update) = if is_v2 {
            // v2: DBから参加者を取得し、embed用のテキストも作成
            debug!("v2募集: DBから参加者を取得します");

            let participants = RecruitmentParticipantEntity::find()
                .filter(ParticipantColumn::RecruitmentId.eq(existing_recruitment.id))
                .all(&txn)
                .await
                .map_err(types::AppError::Database)?;

            // ユニークなユーザーIDを取得（重複排除）
            let unique_user_ids: std::collections::HashSet<i64> =
                participants.iter().map(|p| p.user_id).collect();
            let participant_count = unique_user_ids.len();

            // 参加者メンション（通知用）
            let mentions_str = unique_user_ids
                .iter()
                .map(|user_id| format!("<@{user_id}>"))
                .collect::<Vec<_>>()
                .join(" ");

            // 参加者一覧テキストを作成（embed用）
            let participants_text = create_participants_text_for_v2(
                &recruitment_data.battle_style_name,
                &participants,
                &element_emojis,
            );

            // ドメインモデルでEmbedを作成
            let embed_content = EmbedContent::new()
                .with_title("参加者一覧")
                .with_description(&participants_text)
                .with_footer(format!("参加者数: {participant_count}人"))
                .with_color(0x0099ff);

            (mentions_str, embed_content)
        } else {
            // v1: リアクションから参加者を取得
            debug!("v1募集: リアクションから参加者を取得します");
            let mut participant_ids = std::collections::HashSet::new();
            for reaction in &message.reactions {
                let users = channel_id_obj
                    .reaction_users(
                        http,
                        message_id_obj,
                        reaction.reaction_type.clone(),
                        Some(100),
                        None,
                    )
                    .await?;

                for user in users {
                    if !user.bot {
                        participant_ids.insert(user.id);
                    }
                }
            }

            let mentions_str = participant_ids
                .into_iter()
                .map(|user_id| format!("<@{user_id}>"))
                .collect::<Vec<_>>()
                .join(" ");

            // v1は新規作成用のembedをそのまま使用（ドメインモデル）
            (mentions_str, recruitment_data.embed_content.clone())
        };

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

        // 6. Discordのメッセージを更新（ドメインモデルを使用）
        let edit_content = MessageContent::new()
            .with_text(&recruitment_data.message_content)
            .with_embed(embed_for_update);

        channel_id_obj
            .edit_message(http, message_id_obj, to_edit_message(&edit_content))
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

        // 返信形式で送信を試み、失敗時は文脈情報を付加して通常メッセージとして送信
        send_message_with_optional_reply(
            http,
            channel_id_obj,
            message_id_obj,
            update_notification,
            Some("募集内容変更通知".to_string()),
        )
        .await?;

        // 8. 出発日時が変更された場合、既存の通知を削除して新しい通知を作成
        if event_date.is_some() {
            // 既存の通知を削除
            notification_service
                .delete_recruitment_notifications(&txn, existing_recruitment.id)
                .await?;

            // 新しい通知を登録（5分前とちょうどの時刻）
            notification_service
                .create_recruitment_departure_notification(
                    &txn,
                    new_expiry_date,
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

/// v2募集用の参加者一覧テキストを作成する
///
/// # 引数
/// * `battle_style_name` - 攻略方法の名前
/// * `participants` - 参加者一覧（DBから取得）
/// * `element_emojis` - 属性絵文字
fn create_participants_text_for_v2(
    battle_style_name: &str,
    participants: &[crate::models::entities::worker::recruitment_participants::Model],
    element_emojis: &crate::services::guild_environment_service::ElementEmojis,
) -> String {
    use crate::types::{ALL_ELEMENTS_EMOJI, ELEMENT_NAMES, SIMPLE_JOIN_EMOJI};
    use std::collections::HashMap;

    // 属性IDごとに参加者をグループ化（Noneは0として扱う）
    let mut participants_by_element: HashMap<i32, Vec<u64>> = HashMap::new();
    for participant in participants {
        let element_id = participant.element_id.unwrap_or(0);
        participants_by_element
            .entry(element_id)
            .or_default()
            .push(participant.user_id as u64);
    }

    let mut text = String::new();

    // 6属性の場合
    if battle_style_name == "6属性" {
        let emojis_array = element_emojis.as_array();
        for (idx, (emoji, name)) in emojis_array.iter().zip(ELEMENT_NAMES.iter()).enumerate() {
            let element_id = (idx + 1) as i32;
            if let Some(user_ids) = participants_by_element.get(&element_id) {
                let user_mentions: Vec<String> =
                    user_ids.iter().map(|&uid| format!("<@{uid}>")).collect();
                text.push_str(&format!(
                    "{} {}: {}\n",
                    emoji,
                    name,
                    user_mentions.join(" ")
                ));
            } else {
                text.push_str(&format!("{emoji} {name}: なし\n"));
            }
        }

        // 全属性可能（element_id = 0）
        if let Some(user_ids) = participants_by_element.get(&0) {
            let user_mentions: Vec<String> =
                user_ids.iter().map(|&uid| format!("<@{uid}>")).collect();
            text.push_str(&format!(
                "{} 全属性可能: {}\n",
                ALL_ELEMENTS_EMOJI,
                user_mentions.join(" ")
            ));
        } else {
            text.push_str(&format!("{ALL_ELEMENTS_EMOJI} 全属性可能: なし\n"));
        }
    } else {
        // シンプル参加の場合（element_id = null）
        if let Some(user_ids) = participants_by_element.get(&0) {
            let user_mentions: Vec<String> =
                user_ids.iter().map(|&uid| format!("<@{uid}>")).collect();
            text.push_str(&format!(
                "{} 参加: {}\n",
                SIMPLE_JOIN_EMOJI,
                user_mentions.join(" ")
            ));
        } else {
            text.push_str(&format!("{SIMPLE_JOIN_EMOJI} 参加: なし\n"));
        }
    }

    if text.is_empty() {
        "現在参加者はいません。".to_string()
    } else {
        text
    }
}
