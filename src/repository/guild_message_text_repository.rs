use crate::models::entities::guild_master::guild_message_texts;
use async_trait::async_trait;
use sea_orm::DbErr;

/// ギルド固有メッセージテキストリポジトリの抽象インターフェース
#[async_trait]
pub trait GuildMessageTextRepository: Send + Sync {
    /// ギルドIDとメッセージIDでメッセージテキストを取得
    async fn get_by_guild_and_id<'c, C>(
        &self,
        db: &'c C,
        guild_id: i64,
        message_id: &str,
    ) -> Result<Option<guild_message_texts::Model>, DbErr>
    where
        C: sea_orm::ConnectionTrait;
}
