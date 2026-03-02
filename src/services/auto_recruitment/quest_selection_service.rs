use crate::repository::auto_recruitment::{AutoRecruitmentRepository, UserDesiredQuestRepository};
use crate::types::{AppError, Result};
use sea_orm::DatabaseTransaction;
use tracing::info;

/// クエスト選択更新サービス
///
/// 自動募集設定の存在確認と、ユーザー希望クエストの全置換を担当する。
pub struct QuestSelectionService<AR, UQ>
where
    AR: AutoRecruitmentRepository,
    UQ: UserDesiredQuestRepository,
{
    auto_recruitment_repo: AR,
    user_desired_quest_repo: UQ,
}

impl<AR, UQ> QuestSelectionService<AR, UQ>
where
    AR: AutoRecruitmentRepository,
    UQ: UserDesiredQuestRepository,
{
    pub fn new(auto_recruitment_repo: AR, user_desired_quest_repo: UQ) -> Self {
        Self {
            auto_recruitment_repo,
            user_desired_quest_repo,
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

    /// ユーザーの希望クエストを全置換
    ///
    /// 既存レコードを削除した後、`battle_style_id=0`（属性指定なし）で再登録する。
    pub async fn replace_user_desired_quests(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        user_id: i64,
        quest_ids: &[i32],
    ) -> Result<()> {
        self.user_desired_quest_repo
            .delete_all_by_user(txn, guild_id, user_id)
            .await?;

        for quest_id in quest_ids {
            self.user_desired_quest_repo
                .create(txn, guild_id, user_id, *quest_id, 0)
                .await?;
        }

        info!(
            guild_id = guild_id,
            user_id = user_id,
            quest_ids = ?quest_ids,
            "ユーザー希望クエストを更新しました"
        );

        Ok(())
    }
}
