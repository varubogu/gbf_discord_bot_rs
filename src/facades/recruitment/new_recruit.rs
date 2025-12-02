use crate::infrastructure::database::container::RepositoryContainer;
use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::repository::database::battle_style_repository::SeaOrmBattleStyleRepository;
use crate::repository::database::quest_repository::SeaOrmQuestRepository;
use crate::repository::database::schedule::{NotificationRelBattleRecruitmentRepository, NotificationRepository};
use crate::services::recruitment::new;
use crate::types;
use crate::types::PoiseContext;
use chrono::{DateTime, Duration, Utc};
use sea_orm::TransactionTrait;
use tracing::{debug, info, instrument};

/// 新しい募集を開始する
#[instrument(level = "debug", skip(ctx))]
pub async fn new_recruitment(
    ctx: &PoiseContext<'_>,
    quest_alias: &str,
    battle_style_id: Option<i32>,
    event_date: Option<DateTime<Utc>>,
) -> types::Result<()> {
    info!("BattleRecruitmentFacade::new_recruitment - 新しい募集を開始します");
    let app_state = &ctx.data().app_state;
    let conn = app_state.db();
    let txn = conn.begin().await?;

    // Discord固有情報を取得
    let guild_id = ctx.guild_id().map(|id| id.get()).unwrap_or(0);

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id as i64).await?;
    let channel_id = ctx.channel_id().get();

    let result = async {
        // RepositoryContainerとRepositoryの取得
        let repos = RepositoryContainer::new(conn);
        let battle_recruitment_repo = repos.battle_recruitment();
        let quest_repository = SeaOrmQuestRepository::new(conn.clone());
        let battle_style_repository = SeaOrmBattleStyleRepository::new(conn.clone());

        // 1. 募集データ作成（QuestRepository, BattleStyleRepositoryを使用）
        let recruitment_data =
            new::create_recruitment_data(
                &quest_repository,
                &battle_style_repository,
                quest_alias,
                battle_style_id,
                channel_id,
                guild_id,
                event_date
            ).await?;

        // 2. メッセージ送信
        let message_id = new::send_recruitment_message(ctx, &recruitment_data).await?;

        // 3. リアクション追加
        new::add_recruitment_reactions(ctx, message_id, &recruitment_data.reactions).await?;

        // 4. データ保存
        let recruitment = new::save_recruitment(&txn, battle_recruitment_repo, &recruitment_data, message_id).await?;

        // 5. 出発時刻の通知を登録（出発5分前）
        let notification_repo = NotificationRepository::new(conn.clone());
        let notify_time = recruitment_data.expiry_date - Duration::minutes(5);

        debug!(
            expiry_date = %recruitment_data.expiry_date,
            notify_time = %notify_time,
            "募集の出発通知を登録します"
        );

        let notification = notification_repo
            .create_with_txn(
                &txn,
                notify_time,
                guild_id as i64,
                channel_id as i64,
                "MSG00033".to_string(),
            )
            .await?;

        info!("募集の出発通知を登録しました");

        // 6. 通知と募集のリレーションを作成
        let rel_repo = NotificationRelBattleRecruitmentRepository::new(conn.clone());
        rel_repo
            .create_with_txn(&txn, recruitment.id, notification.id)
            .await?;

        info!("募集と通知のリレーションを登録しました");

        Ok(())
    }
    .await;

    match result {
        Ok(_) => {
            txn.commit().await?;
            Ok(())
        }
        Err(e) => {
            txn.rollback().await?;
            Err(e)
        }
    }
}
