//! 自動募集マッチング属性人数リポジトリの抽象インターフェース

use crate::models::entities::guild_master::auto_recruitment_match_rule_quotas;
use crate::types::Result;
use async_trait::async_trait;
use sea_orm::DatabaseTransaction;

#[async_trait]
pub trait AutoRecruitmentMatchRuleQuotaRepository: Send + Sync {
    /// ギルド内の全属性人数設定を取得
    async fn find_all_by_guild(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
    ) -> Result<Vec<auto_recruitment_match_rule_quotas::Model>>;

    /// ギルド・クエスト単位の属性人数設定を取得
    async fn find_by_guild_and_quest(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        quest_id: i32,
    ) -> Result<Vec<auto_recruitment_match_rule_quotas::Model>>;
}
