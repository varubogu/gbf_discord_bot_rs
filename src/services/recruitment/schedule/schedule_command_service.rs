use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::repository::database::schedule::BattleRecruitmentScheduleRepository;
use crate::types::Result;
use sea_orm::{DatabaseConnection, DatabaseTransaction};

/// スケジュール操作サービス（削除・有効/無効切替）
///
/// Facade層から呼び出され、Repository層への具体的なアクセスを集約する。
pub struct ScheduleCommandService;

impl ScheduleCommandService {
    pub fn new() -> Self {
        Self
    }

    /// スケジュールを削除
    pub async fn delete_schedule(
        &self,
        txn: &DatabaseTransaction,
        db: &DatabaseConnection,
        schedule_id: i32,
    ) -> Result<()> {
        let repo = BattleRecruitmentScheduleRepository::new();

        // RLSのため、対象スケジュールのギルドIDでセッション変数を設定
        if let Some((model, _)) = repo.find_by_id(db, schedule_id).await? {
            set_current_guild_id(txn, model.guild_id).await?;
        }

        repo.delete_with_txn(txn, schedule_id).await?;
        Ok(())
    }

    /// スケジュールの有効/無効を現在値から反転
    pub async fn toggle_schedule_enabled(
        &self,
        txn: &DatabaseTransaction,
        db: &DatabaseConnection,
        schedule_id: i32,
    ) -> Result<()> {
        let repo = BattleRecruitmentScheduleRepository::new();

        // 現在の状態とギルドIDを取得
        let (guild_id, new_enabled) =
            if let Some((model, _)) = repo.find_by_id(db, schedule_id).await? {
                (model.guild_id, !model.is_enabled)
            } else {
                return Err(crate::types::AppError::NotFound(format!(
                    "スケジュールID {} が見つかりません",
                    schedule_id
                )));
            };

        // RLS設定
        set_current_guild_id(txn, guild_id).await?;

        // 反転適用
        repo.toggle_enabled_with_txn(txn, schedule_id, new_enabled)
            .await?;
        Ok(())
    }
}

impl Default for ScheduleCommandService {
    fn default() -> Self {
        Self::new()
    }
}
