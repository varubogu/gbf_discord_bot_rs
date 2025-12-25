use crate::repository::database::schedule::BattleRecruitmentScheduleRepository;
use crate::types::Result;
use sea_orm::DatabaseTransaction;

/// スケジュール操作サービス（削除・有効/無効切替）
///
/// Facade層から呼び出され、Repository層への具体的なアクセスを集約する。
/// RLS設定はFacade層で行われるため、このService層では行わない。
pub struct ScheduleCommandService;

impl ScheduleCommandService {
    pub fn new() -> Self {
        Self
    }

    /// スケジュールを削除
    ///
    /// RLS設定は呼び出し元のFacade層で既に行われている前提
    pub async fn delete_schedule(&self, txn: &DatabaseTransaction, schedule_id: i32) -> Result<()> {
        let repo = BattleRecruitmentScheduleRepository::new();
        repo.delete_with_txn(txn, schedule_id).await?;
        Ok(())
    }

    /// スケジュールの有効/無効を現在値から反転
    ///
    /// RLS設定は呼び出し元のFacade層で既に行われている前提
    pub async fn toggle_schedule_enabled(
        &self,
        txn: &DatabaseTransaction,
        schedule_id: i32,
    ) -> Result<()> {
        let repo = BattleRecruitmentScheduleRepository::new();

        // 現在の状態を取得（RLS設定済みのトランザクションを使用）
        let new_enabled = if let Some((model, _)) = repo.find_by_id(txn, schedule_id).await? {
            !model.is_enabled
        } else {
            return Err(crate::types::AppError::NotFound(format!(
                "スケジュールID {schedule_id} が見つかりません"
            )));
        };

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
