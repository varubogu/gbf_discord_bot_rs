use crate::infrastructure::database::session::set_current_guild_id;
use crate::services::recruitment::schedule::{
    ScheduleCommandService, ScheduleCreateService, ScheduleCreationResult,
};
use crate::services::schedule::schedule_query_service::{ScheduleListItem, ScheduleQueryService};
use crate::services::timezone_service::TimezoneService;
use crate::types::app_state::AppState;
use crate::types::{AppError, Result};
use sea_orm::TransactionTrait;
use std::sync::Arc;
use tracing::{error, info};

/// 定期募集スケジュールファサード
///
/// 定期募集スケジュールの作成・更新・削除などのユースケースを管理するファサード。
/// トランザクション境界の管理と複数サービスの協調を担当。
pub struct RecruitmentScheduleFacade {
    app_state: Arc<AppState>,
}

impl RecruitmentScheduleFacade {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }

    /// 定期募集スケジュールを作成
    ///
    /// # 引数
    /// - `guild_id`: ギルドID
    /// - `user_id`: ユーザーID
    /// - `name`: スケジュール名
    /// - `quest_alias`: クエスト名またはエイリアス
    /// - `quest_start_time`: クエスト開始時刻（ローカル時刻、HH:MM形式）
    /// - `days`: 対象曜日文字列
    /// - `recruit_start_time`: 募集開始時刻（ローカル時刻、HH:MM形式）
    /// - `battle_style_id`: バトルスタイルID（省略可）
    /// - `recruit_day_offset`: 募集開始日オフセット（省略時は自動判定）
    /// - `note`: 備考（省略可）
    ///
    /// # 戻り値
    /// スケジュール作成結果
    ///
    /// # トランザクション管理
    /// このメソッドはトランザクションを開始・コミット・ロールバックを管理します。
    #[allow(clippy::too_many_arguments)]
    pub async fn create_recruitment_schedule(
        &self,
        guild_id: u64,
        user_id: u64,
        name: String,
        quest_alias: &str,
        quest_start_time: &str,
        days: &str,
        recruit_start_time: &str,
        battle_style_id: Option<i32>,
        recruit_day_offset: Option<i32>,
        note: Option<String>,
        dismissal_times: Option<String>,
    ) -> Result<ScheduleCreationResult> {
        info!(
            guild_id = guild_id,
            user_id = user_id,
            name = %name,
            "定期募集スケジュール作成を開始します"
        );

        // トランザクション開始（Facade層の責務）
        let conn = self.app_state.guild_db();
        let txn = conn.begin().await?;

        // RLSポリシーのためにセッション変数を設定
        set_current_guild_id(&txn, guild_id as i64).await?;

        let result = async {
            // 1. タイムゾーン取得Service
            let timezone_repo = self.app_state.repositories.guild_settings;
            let timezone_service = TimezoneService::new(timezone_repo);
            let timezone = timezone_service
                .get_guild_timezone_with_txn(&txn, guild_id as i64)
                .await?;

            info!(
                guild_id = guild_id,
                timezone = %timezone,
                "ギルドのタイムゾーンを取得しました"
            );

            // 2. スケジュール作成Service
            let schedule_service = ScheduleCreateService::new(
                self.app_state.repositories.quest,
                self.app_state.repositories.battle_style,
                self.app_state.repositories.guild_channel,
                self.app_state.repositories.battle_recruitment_schedule,
                self.app_state.repositories.scheduled_task,
                self.app_state.repositories.scheduled_task_recurring,
                self.app_state
                    .repositories
                    .battle_recruitment_schedule_dismissal,
            );
            let schedule_data = schedule_service
                .create_schedule(
                    &txn,
                    guild_id as i64,
                    user_id as i64,
                    name,
                    quest_alias,
                    quest_start_time,
                    days,
                    recruit_start_time,
                    battle_style_id,
                    recruit_day_offset,
                    note,
                    dismissal_times,
                    timezone,
                )
                .await?;

            Ok::<_, AppError>(schedule_data)
        }
        .await;

        // 結果に応じてcommit/rollback（Facade層の責務）
        match result {
            Ok(schedule_data) => {
                txn.commit().await?;
                info!(
                    schedule_id = schedule_data.schedule_id,
                    guild_id = guild_id,
                    "定期募集スケジュールの作成に成功しました"
                );
                Ok(schedule_data)
            }
            Err(e) => {
                txn.rollback().await?;
                error!(
                    error = %e,
                    guild_id = guild_id,
                    "定期募集スケジュールの作成に失敗しました"
                );
                Err(e)
            }
        }
    }

    /// 募集スケジュール一覧取得（Facade: トランザクション境界の管理）
    pub async fn list_recruitment_schedules(
        &self,
        guild_id: i64,
        user_id: i64,
        show_all: bool,
    ) -> Result<Vec<ScheduleListItem>> {
        let conn = self.app_state.guild_db();
        let txn = conn.begin().await?;

        // RLSのためにセッション変数を設定
        set_current_guild_id(&txn, guild_id).await?;

        let result = async {
            // タイムゾーン取得
            let timezone_service = TimezoneService::new(self.app_state.repositories.guild_settings);
            let tz = timezone_service
                .get_guild_timezone_with_txn(&txn, guild_id)
                .await?;

            // 一覧取得
            let service = ScheduleQueryService::new(
                self.app_state.repositories.battle_recruitment_schedule,
                self.app_state.repositories.quest,
                self.app_state
                    .repositories
                    .battle_recruitment_schedule_dismissal,
                self.app_state.repositories.notification,
            );
            let list = service
                .get_schedule_list(&txn, conn, guild_id, user_id, show_all, tz)
                .await?;
            Ok::<_, AppError>(list)
        }
        .await;

        match result {
            Ok(list) => {
                txn.commit().await?;
                Ok(list)
            }
            Err(e) => {
                txn.rollback().await?;
                Err(e)
            }
        }
    }

    /// スケジュール削除
    ///
    /// # 引数
    /// - `guild_id`: ギルドID
    /// - `schedule_id`: スケジュールID
    /// - `user_id`: コマンド実行ユーザーID
    /// - `is_admin`: 管理者権限の有無
    ///
    /// # 権限チェック
    /// - 作成者本人または管理者のみが削除可能
    pub async fn delete_recruitment_schedule(
        &self,
        guild_id: i64,
        schedule_id: i32,
        user_id: i64,
        is_admin: bool,
    ) -> Result<()> {
        let conn = self.app_state.guild_db();
        let txn = conn.begin().await?;

        // RLSのためにセッション変数を設定
        set_current_guild_id(&txn, guild_id).await?;

        let result = async {
            let service = ScheduleCommandService::new(
                self.app_state.repositories.battle_recruitment_schedule,
                self.app_state.repositories.scheduled_task,
                self.app_state.repositories.scheduled_task_recurring,
            );
            service
                .assert_schedule_deletable_by(&txn, schedule_id, user_id, is_admin)
                .await?;
            service.delete_schedule(&txn, schedule_id).await?;
            Ok::<_, AppError>(())
        }
        .await;

        match result {
            Ok(_) => {
                txn.commit().await?;
                info!(
                    schedule_id,
                    guild_id, user_id, "募集スケジュールを削除しました"
                );
                Ok(())
            }
            Err(e) => {
                error!(error = %e, schedule_id, guild_id, user_id, "募集スケジュールの削除に失敗しました");
                txn.rollback().await?;
                Err(e)
            }
        }
    }

    /// スケジュールの有効/無効切替
    ///
    /// # 引数
    /// - `guild_id`: ギルドID
    /// - `schedule_id`: スケジュールID
    /// - `user_id`: コマンド実行ユーザーID
    /// - `is_admin`: 管理者権限の有無
    ///
    /// # 権限チェック
    /// - 作成者本人または管理者のみが切り替え可能
    pub async fn toggle_recruitment_schedule(
        &self,
        guild_id: i64,
        schedule_id: i32,
        user_id: i64,
        is_admin: bool,
    ) -> Result<()> {
        let conn = self.app_state.guild_db();
        let txn = conn.begin().await?;

        // RLSのためにセッション変数を設定
        set_current_guild_id(&txn, guild_id).await?;

        let result = async {
            let service = ScheduleCommandService::new(
                self.app_state.repositories.battle_recruitment_schedule,
                self.app_state.repositories.scheduled_task,
                self.app_state.repositories.scheduled_task_recurring,
            );
            service
                .assert_schedule_operable_by(&txn, schedule_id, user_id, is_admin, "切り替え")
                .await?;

            // 現在の状態を取得
            let is_enabled = service
                .get_schedule_enabled_status(&txn, schedule_id)
                .await?;

            // 状態に応じて有効化/無効化を呼び分け
            if is_enabled {
                service.disable_schedule(&txn, schedule_id).await?;
            } else {
                service.enable_schedule(&txn, schedule_id).await?;
            }

            Ok::<_, AppError>(())
        }
        .await;

        match result {
            Ok(_) => {
                txn.commit().await?;
                info!(
                    schedule_id,
                    guild_id, user_id, "募集スケジュールの有効/無効を切り替えました"
                );
                Ok(())
            }
            Err(e) => {
                error!(error = %e, schedule_id, guild_id, user_id, "募集スケジュールの切替に失敗しました");
                txn.rollback().await?;
                Err(e)
            }
        }
    }
}
