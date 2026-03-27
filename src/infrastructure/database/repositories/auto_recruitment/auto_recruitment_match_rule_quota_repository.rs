//! 自動募集マッチング属性人数リポジトリのSeaORM実装

use crate::models::entities::guild_master::auto_recruitment_match_rule_quotas;
use crate::repository::auto_recruitment::AutoRecruitmentMatchRuleQuotaRepository as AutoRecruitmentMatchRuleQuotaRepositoryTrait;
use crate::types::Result;
use async_trait::async_trait;
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder};
use tracing::{debug, error};

#[derive(Debug, Clone, Copy)]
pub struct SeaOrmAutoRecruitmentMatchRuleQuotaRepository;

#[async_trait]
impl AutoRecruitmentMatchRuleQuotaRepositoryTrait
    for SeaOrmAutoRecruitmentMatchRuleQuotaRepository
{
    async fn find_all_by_guild(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
    ) -> Result<Vec<auto_recruitment_match_rule_quotas::Model>> {
        debug!(guild_id, "ギルドの全属性人数設定を取得します");

        let result = auto_recruitment_match_rule_quotas::Entity::find()
            .filter(auto_recruitment_match_rule_quotas::Column::GuildId.eq(guild_id))
            .order_by_asc(auto_recruitment_match_rule_quotas::Column::QuestId)
            .order_by_asc(auto_recruitment_match_rule_quotas::Column::SortOrder)
            .order_by_asc(auto_recruitment_match_rule_quotas::Column::BattleStyleId)
            .all(txn)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id, "属性人数設定の取得に失敗しました");
                e
            })?;

        debug!(
            guild_id,
            count = result.len(),
            "ギルドの全属性人数設定を取得しました"
        );
        Ok(result)
    }

    async fn find_by_guild_and_quest(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        quest_id: i32,
    ) -> Result<Vec<auto_recruitment_match_rule_quotas::Model>> {
        debug!(guild_id, quest_id, "属性人数設定を取得します");

        let result = auto_recruitment_match_rule_quotas::Entity::find()
            .filter(auto_recruitment_match_rule_quotas::Column::GuildId.eq(guild_id))
            .filter(auto_recruitment_match_rule_quotas::Column::QuestId.eq(quest_id))
            .order_by_asc(auto_recruitment_match_rule_quotas::Column::SortOrder)
            .order_by_asc(auto_recruitment_match_rule_quotas::Column::BattleStyleId)
            .all(txn)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id, quest_id, "属性人数設定の取得に失敗しました");
                e
            })?;

        Ok(result)
    }
}

impl SeaOrmAutoRecruitmentMatchRuleQuotaRepository {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SeaOrmAutoRecruitmentMatchRuleQuotaRepository {
    fn default() -> Self {
        Self::new()
    }
}
