use std::sync::Arc;
use tracing::error;

use crate::models::battle_recruitment::BattleRecruitment;
use crate::repository::BattleRecruitmentRepository;

/// 募集情報取得サービス
pub struct GetRecruitmentService {
    battle_recruitment_repo: Arc<dyn BattleRecruitmentRepository>,
}

impl GetRecruitmentService {
    /// 依存性注入パターンに従ったコンストラクタ
    pub fn new(battle_recruitment_repo: Arc<dyn BattleRecruitmentRepository>) -> Self {
        Self {
            battle_recruitment_repo,
        }
    }

    /// メッセージIDから募集情報を取得する
    pub async fn get_by_message(
        &self,
        guild_id: u64,
        channel_id: u64,
        message_id: u64,
    ) -> Result<Option<BattleRecruitment>, String> {
        match self
            .battle_recruitment_repo
            .get_by_message(guild_id as i64, channel_id as i64, message_id as i64)
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
    pub async fn get_by_id(&self, recruitment_id: i32) -> Result<Option<BattleRecruitment>, String> {
        match self.battle_recruitment_repo.get_by_id(recruitment_id).await {
            Ok(recruitment) => Ok(recruitment),
            Err(e) => {
                error!("Error getting recruitment by id: {:?}", e);
                Err(format!("Failed to get recruitment: {}", e))
            }
        }
    }
}