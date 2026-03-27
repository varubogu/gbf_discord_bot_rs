//! 自動募集マッチングルールリポジトリの抽象インターフェース

use crate::models::entities::guild_master::auto_recruitment_match_rules;
use crate::types::Result;
use async_trait::async_trait;
use sea_orm::DatabaseTransaction;

#[async_trait]
pub trait AutoRecruitmentMatchRuleRepository: Send + Sync {
    /// ギルド内の全マッチングルールを取得
    async fn find_all_by_guild(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
    ) -> Result<Vec<auto_recruitment_match_rules::Model>>;

    /// ギルド・クエスト単位のマッチングルールを取得
    async fn find_by_guild_and_quest(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        quest_id: i32,
    ) -> Result<Option<auto_recruitment_match_rules::Model>>;
}
