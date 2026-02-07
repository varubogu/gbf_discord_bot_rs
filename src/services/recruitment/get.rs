use tracing::error;

use crate::models::battle_recruitments::BattleRecruitments;
use crate::repository::battle_recruitments_repository::BattleRecruitmentsRepository;
use crate::types::discord::{DiscordChannelId, DiscordGuildId, DiscordMessageId};

/// 募集情報取得サービス
pub struct GetRecruitmentService<R: BattleRecruitmentsRepository> {
    battle_recruitment_repo: R,
}

impl<R: BattleRecruitmentsRepository> GetRecruitmentService<R> {
    /// 依存性注入パターンに従ったコンストラクタ
    pub fn new(battle_recruitment_repo: R) -> Self {
        Self {
            battle_recruitment_repo,
        }
    }

    /// メッセージIDから募集情報を取得する（通常接続版）
    pub async fn get_by_message(
        &self,
        db: &sea_orm::DatabaseConnection,
        guild_id: u64,
        channel_id: u64,
        message_id: u64,
    ) -> Result<Option<BattleRecruitments>, String> {
        // u64をドメイン型に変換
        match self
            .battle_recruitment_repo
            .get_by_message_with_db(
                db,
                DiscordGuildId::new(guild_id),
                DiscordChannelId::new(channel_id),
                DiscordMessageId::new(message_id),
            )
            .await
        {
            Ok(recruitment) => Ok(recruitment),
            Err(e) => {
                error!("Error getting recruitment by message: {:?}", e);
                Err(format!("Failed to get recruitment: {e}"))
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
