use crate::models::battle_recruitments::BattleRecruitments;
use crate::repository::BattleRecruitmentsRepository;
use crate::types::Result;
use sea_orm::DatabaseTransaction;

/// 募集一覧取得サービス
///
/// 現在募集中の募集レコード取得を担当する。
pub struct RecruitListService<R>
where
    R: BattleRecruitmentsRepository,
{
    battle_recruitments_repo: R,
}

impl<R> RecruitListService<R>
where
    R: BattleRecruitmentsRepository,
{
    pub fn new(battle_recruitments_repo: R) -> Self {
        Self {
            battle_recruitments_repo,
        }
    }

    /// ギルドの募集中募集を取得
    pub async fn get_active_recruitments(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
    ) -> Result<Vec<BattleRecruitments>> {
        self.battle_recruitments_repo
            .get_active_by_guild_with_txn(txn, guild_id)
            .await
    }
}
