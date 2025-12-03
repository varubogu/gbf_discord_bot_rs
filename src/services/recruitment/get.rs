use std::sync::Arc;
use tracing::error;

use crate::models::battle_recruitments::BattleRecruitments;
use crate::repository::battle_recruitments_repository::BattleRecruitmentsRepository;
use crate::repository::database::battle_recruitments_repository::BattleRecruitmentsRepositoryImpl;

/// 募集情報取得サービス
pub struct GetRecruitmentService {
    battle_recruitment_repo: Arc<BattleRecruitmentsRepositoryImpl>,
}

impl GetRecruitmentService {
    /// 依存性注入パターンに従ったコンストラクタ
    pub fn new(battle_recruitment_repo: Arc<BattleRecruitmentsRepositoryImpl>) -> Self {
        Self {
            battle_recruitment_repo,
        }
    }

    /// メッセージIDから募集情報を取得する
    pub async fn get_by_message<'c, C>(
        &self,
        db: &'c C,
        guild_id: u64,
        channel_id: u64,
        message_id: u64,
    ) -> Result<Option<BattleRecruitments>, String>
    where
        C: sea_orm::ConnectionTrait,
    {
        match self
            .battle_recruitment_repo
            .get_by_message(db, guild_id, channel_id, message_id)
            .await
        {
            Ok(recruitment) => Ok(recruitment),
            Err(e) => {
                error!("Error getting recruitment by message: {:?}", e);
                Err(format!("Failed to get recruitment: {}", e))
            }
        }
    }

    /// 募集IDから募集情報を取得する
    pub async fn get_by_id(
        &self,
        _recruitment_id: i32,
    ) -> Result<Option<BattleRecruitments>, String> {
        // match self.battle_recruitment_repo.get_by_id(recruitment_id).await {
        //     Ok(recruitment) => Ok(recruitment),
        //     Err(e) => {
        //         error!("Error getting recruitment by id: {:?}", e);
        //         Err(format!("Failed to get recruitment: {}", e))
        //     }
        // }
        Ok(None)
    }
}
