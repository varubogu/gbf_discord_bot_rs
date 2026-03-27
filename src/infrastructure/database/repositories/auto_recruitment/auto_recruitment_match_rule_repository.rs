//! 自動募集マッチングルールリポジトリのSeaORM実装

use crate::models::entities::guild_master::auto_recruitment_match_rules;
use crate::repository::auto_recruitment::AutoRecruitmentMatchRuleRepository as AutoRecruitmentMatchRuleRepositoryTrait;
use crate::types::Result;
use async_trait::async_trait;
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use tracing::{debug, error};

#[derive(Debug, Clone, Copy)]
pub struct SeaOrmAutoRecruitmentMatchRuleRepository;

#[async_trait]
impl AutoRecruitmentMatchRuleRepositoryTrait for SeaOrmAutoRecruitmentMatchRuleRepository {
    async fn find_all_by_guild(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
    ) -> Result<Vec<auto_recruitment_match_rules::Model>> {
        debug!(guild_id, "ギルドの全マッチングルールを取得します");

        let result = auto_recruitment_match_rules::Entity::find()
            .filter(auto_recruitment_match_rules::Column::GuildId.eq(guild_id))
            .all(txn)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id, "マッチングルールの取得に失敗しました");
                e
            })?;

        debug!(
            guild_id,
            count = result.len(),
            "ギルドの全マッチングルールを取得しました"
        );
        Ok(result)
    }

    async fn find_by_guild_and_quest(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        quest_id: i32,
    ) -> Result<Option<auto_recruitment_match_rules::Model>> {
        debug!(guild_id, quest_id, "マッチングルールを取得します");

        let result = auto_recruitment_match_rules::Entity::find()
            .filter(auto_recruitment_match_rules::Column::GuildId.eq(guild_id))
            .filter(auto_recruitment_match_rules::Column::QuestId.eq(quest_id))
            .one(txn)
            .await
            .map_err(|e| {
                error!(error = %e, guild_id, quest_id, "マッチングルールの取得に失敗しました");
                e
            })?;

        Ok(result)
    }
}

impl SeaOrmAutoRecruitmentMatchRuleRepository {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SeaOrmAutoRecruitmentMatchRuleRepository {
    fn default() -> Self {
        Self::new()
    }
}
