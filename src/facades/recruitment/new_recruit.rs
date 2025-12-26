use crate::infrastructure::database::container::RepositoryContainer;
use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::repository::database::guild_environment_repository::SeaOrmGuildEnvironmentRepository;
use crate::repository::database::guild_settings_repository::GuildSettingsRepository;
use crate::services::guild_environment_service::GuildEnvironmentService;
use crate::services::recruitment::dismissal_time_parser_service::DismissalTimeParserService;
use crate::services::recruitment::new;
use crate::services::recruitment::role_notification::RoleNotificationService;
use crate::services::schedule::{DismissalManagementService, NotificationManagementService};
use crate::services::timezone_service::TimezoneService;
use crate::types;
use crate::types::PoiseContext;
use chrono::{DateTime, Utc};
use sea_orm::TransactionTrait;
use std::sync::Arc;
use tracing::{debug, info, instrument};

/// 新しい募集を開始する
///
/// # 引数
/// * `use_buttons` - ボタンを使用する場合は true、リアクションを使用する場合は false
///
/// # 戻り値
/// (message_id, reactions) - メッセージIDとリアクションのリスト
#[instrument(level = "debug", skip(ctx))]
pub async fn new_recruitment(
    ctx: &PoiseContext<'_>,
    quest_alias: &str,
    battle_style_id: Option<i32>,
    event_date: Option<DateTime<Utc>>,
    use_buttons: bool,
    dismissal_times: Option<String>,
) -> types::Result<(u64, Vec<poise::serenity_prelude::ReactionType>)> {
    info!("BattleRecruitmentFacade::new_recruitment - 新しい募集を開始します");
    let app_state = &ctx.data().app_state;
    let conn = app_state.guild_db();
    let txn = conn.begin().await?;

    // Discord固有情報を取得
    let guild_id = ctx.guild_id().map(|id| id.get()).unwrap_or(0);

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id as i64).await?;
    let channel_id = ctx.channel_id().get();

    let result = async {
        // RepositoryContainerの取得
        let repos = RepositoryContainer::new();
        let battle_recruitment_repo = repos.battle_recruitment();

        // タイムゾーンを取得
        let timezone_repo = Arc::new(GuildSettingsRepository::new());
        let timezone_service = TimezoneService::new(timezone_repo);
        let timezone = timezone_service.get_guild_timezone(conn, guild_id as i64).await?;

        // 属性絵文字を取得（ギルド固有設定 or デフォルト値）
        let guild_env_repo = Arc::new(SeaOrmGuildEnvironmentRepository::new());
        let guild_env_service = GuildEnvironmentService::new(guild_env_repo);
        let element_emojis = guild_env_service.get_element_emojis(conn, ctx.http(), guild_id as i64).await?;

        // 1. 募集データ作成（Serviceラッパー関数を使用）
        let mut recruitment_data =
            new::create_recruitment_data_with_repos(
                conn,
                &element_emojis,
                quest_alias,
                battle_style_id,
                channel_id,
                guild_id,
                event_date,
                timezone,
            ).await?;

        // 1.5. ロールメンションを取得してメッセージの先頭に追加
        let role_service = RoleNotificationService::new();
        let role_mentions = role_service
            .get_role_mentions(&txn, guild_id as i64, recruitment_data.quest.id)
            .await?;

        if !role_mentions.is_empty() {
            debug!(role_mentions = %role_mentions, "ロールメンションを募集メッセージの先頭に追加します");
            recruitment_data.message_content = format!("{}\n{}", role_mentions, recruitment_data.message_content);
            info!("ロールメンションを募集メッセージに追加しました");
        }

        // 2. メッセージ送信（ボタンまたはリアクション用）
        let message_id = if use_buttons {
            new::send_recruitment_message_with_buttons(ctx, &recruitment_data).await?
        } else {
            new::send_recruitment_message(ctx, &recruitment_data).await?
        };

        // 3. データ保存
        let recruitment = new::save_recruitment(&txn, battle_recruitment_repo, &recruitment_data, message_id).await?;

        // 4. 出発時刻の通知を登録（5分前とちょうどの時刻）
        debug!(
            expiry_date = %recruitment_data.expiry_date,
            "募集の出発通知を登録します"
        );

        let notification_management_service = NotificationManagementService::new();
        notification_management_service
            .create_recruitment_departure_notification(
                &txn,
                recruitment_data.expiry_date,
                guild_id as i64,
                channel_id as i64,
                recruitment.id,
            )
            .await?;

        // 5. 解散時刻を登録（指定されている場合）
        if let Some(ref dismissal_times_str) = dismissal_times {
            debug!(
                dismissal_times = %dismissal_times_str,
                recruitment_id = recruitment.id,
                "解散時刻のパースと登録を開始します"
            );

            // 環境変数から最大日数を取得（デフォルト7日）
            let max_days = std::env::var("DISMISSAL_MAX_DAYS")
                .ok()
                .and_then(|v| v.parse::<i32>().ok())
                .unwrap_or(7);

            // 解散時刻をパース
            let parser = DismissalTimeParserService::new();
            let parsed_dismissal_times = parser.parse(
                dismissal_times_str,
                recruitment_data.expiry_date,
                timezone,
                max_days,
            )?;

            // 解散時刻を含めたメッセージを再生成
            let message_content_with_dismissal = new::create_message_content(
                &txn,
                &recruitment_data.quest.name,
                &recruitment_data.battle_style_name,
                &recruitment_data.expiry_date,
                timezone,
                Some(guild_id as i64),
                Some(&parsed_dismissal_times),
            )
            .await?;

            // ロールメンションがある場合は先頭に追加
            let final_message_content = if !role_mentions.is_empty() {
                format!("{}\n{}", role_mentions, message_content_with_dismissal)
            } else {
                message_content_with_dismissal
            };

            // Discordメッセージを更新
            use poise::serenity_prelude::{ChannelId, MessageId};
            let channel = ChannelId::new(channel_id);
            let msg = MessageId::new(message_id);
            channel
                .edit_message(ctx.http(), msg, poise::serenity_prelude::EditMessage::new().content(final_message_content))
                .await?;

            // 解散時刻を登録
            let dismissal_service = DismissalManagementService::new();
            dismissal_service
                .create_recruitment_dismissals(
                    &txn,
                    recruitment.id,
                    parsed_dismissal_times,
                    recruitment_data.expiry_date,
                    guild_id as i64,
                    channel_id as i64,
                )
                .await?;

            info!(
                recruitment_id = recruitment.id,
                "解散時刻を登録しました"
            );
        }

        // message_idとreactionsを返す
        Ok((message_id, recruitment_data.reactions.clone()))
    }
    .await;

    match result {
        Ok((msg_id, reactions)) => {
            txn.commit().await?;
            Ok((msg_id, reactions))
        }
        Err(e) => {
            txn.rollback().await?;
            Err(e)
        }
    }
}
