use crate::repository::auto_recruitment::{
    AutoRecruitmentParticipantRepository, AutoRecruitmentRepository,
};
use crate::types::{AppError, Result};
use sea_orm::DatabaseTransaction;
use tracing::info;

/// 自動募集の時間選択更新サービス
///
/// 自動募集設定の存在確認と、ユーザーの時間選択の全置換を担当する。
pub struct TimeSelectionService<AR, PR>
where
    AR: AutoRecruitmentRepository,
    PR: AutoRecruitmentParticipantRepository,
{
    auto_recruitment_repo: AR,
    participant_repo: PR,
}

impl<AR, PR> TimeSelectionService<AR, PR>
where
    AR: AutoRecruitmentRepository,
    PR: AutoRecruitmentParticipantRepository,
{
    pub fn new(auto_recruitment_repo: AR, participant_repo: PR) -> Self {
        Self {
            auto_recruitment_repo,
            participant_repo,
        }
    }

    /// 自動募集設定の存在確認
    pub async fn ensure_auto_recruitment_exists(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
    ) -> Result<()> {
        self.auto_recruitment_repo
            .find_by_guild_id(txn, guild_id)
            .await?
            .ok_or_else(|| AppError::Business {
                message: "このギルドには自動募集が登録されていません".to_string(),
            })?;

        Ok(())
    }

    /// ユーザーの時間選択を全置換
    pub async fn replace_user_time_selection(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        user_id: i64,
        month: i32,
        day: i32,
        hours: &[i32],
    ) -> Result<()> {
        self.participant_repo
            .delete_all_by_user_date(txn, guild_id, user_id, month, day)
            .await?;

        for hour in hours {
            self.participant_repo
                .create(txn, guild_id, user_id, month, day, *hour)
                .await?;
        }

        info!(
            guild_id = guild_id,
            user_id = user_id,
            month = month,
            day = day,
            hours = ?hours,
            "ユーザーの時間選択を更新しました"
        );

        Ok(())
    }
}
