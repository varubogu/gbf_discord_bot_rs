use crate::models::battle_recruitments::BattleRecruitments;
use crate::models::entities::battle_styles;
use crate::repository::database::battle_style_repository::{
    BattleStyleRepository, SeaOrmBattleStyleRepository,
};
use crate::repository::BattleRecruitmentsRepository;
use crate::types::Result;
use sea_orm::DatabaseTransaction;
use tracing::debug;

/// 募集情報クエリService
/// 募集情報とBattleStyleの取得を担当
pub struct RecruitmentQueryService;

impl RecruitmentQueryService {
    pub fn new() -> Self {
        Self
    }

    /// メッセージIDから募集情報を取得
    pub async fn get_recruitment_by_message(
        &self,
        txn: &DatabaseTransaction,
        guild_id: u64,
        channel_id: u64,
        message_id: u64,
    ) -> Result<Option<BattleRecruitments>> {
        use crate::infrastructure::database::container::RepositoryContainer;

        let repos = RepositoryContainer::new();
        let battle_recruitment_repo = repos.battle_recruitment();

        let recruitment = battle_recruitment_repo
            .get_by_message_with_txn(txn, guild_id, channel_id, message_id)
            .await?;

        debug!(
            guild_id = guild_id,
            channel_id = channel_id,
            message_id = message_id,
            found = recruitment.is_some(),
            "メッセージIDから募集情報を取得しました"
        );

        Ok(recruitment)
    }

    /// BattleStyleをIDで取得
    pub async fn get_battle_style_by_id(
        &self,
        txn: &DatabaseTransaction,
        battle_style_id: i32,
    ) -> Result<Option<battle_styles::Model>> {
        let battle_style_repo = SeaOrmBattleStyleRepository::new();
        let battle_style = battle_style_repo.get_by_id(txn, battle_style_id).await?;

        debug!(
            battle_style_id = battle_style_id,
            found = battle_style.is_some(),
            "BattleStyle情報を取得しました"
        );

        Ok(battle_style)
    }
}
