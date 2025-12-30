use crate::types::Result;
use async_trait::async_trait;

/// ギルドクエスト無効化リポジトリの抽象インターフェース
#[async_trait]
pub trait GuildQuestDisableRepository: Send + Sync {
    /// クエストを無効化（レコード追加）
    ///
    /// # Arguments
    /// * `db` - DatabaseConnection または DatabaseTransaction
    /// * `guild_id` - ギルドID
    /// * `quest_id` - クエストID
    async fn disable_quest<'c, C>(&self, db: &'c C, guild_id: i64, quest_id: i32) -> Result<()>
    where
        C: sea_orm::ConnectionTrait;

    /// クエストを有効化（レコード削除）
    ///
    /// # Arguments
    /// * `db` - DatabaseConnection または DatabaseTransaction
    /// * `guild_id` - ギルドID
    /// * `quest_id` - クエストID
    async fn enable_quest<'c, C>(&self, db: &'c C, guild_id: i64, quest_id: i32) -> Result<()>
    where
        C: sea_orm::ConnectionTrait;

    /// 無効化されているクエストIDのリストを取得
    ///
    /// # Arguments
    /// * `db` - DatabaseConnection または DatabaseTransaction
    /// * `guild_id` - ギルドID
    async fn get_disabled_quest_ids<'c, C>(&self, db: &'c C, guild_id: i64) -> Result<Vec<i32>>
    where
        C: sea_orm::ConnectionTrait;

    /// 指定されたクエストが無効化されているか確認
    ///
    /// # Arguments
    /// * `db` - DatabaseConnection または DatabaseTransaction
    /// * `guild_id` - ギルドID
    /// * `quest_id` - クエストID
    async fn is_disabled<'c, C>(&self, db: &'c C, guild_id: i64, quest_id: i32) -> Result<bool>
    where
        C: sea_orm::ConnectionTrait;
}
