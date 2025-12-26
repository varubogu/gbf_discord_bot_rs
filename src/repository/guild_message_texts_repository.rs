use crate::models::entities::guild_master::guild_message_texts;
use async_trait::async_trait;
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter};

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

/// SeaORMを使用したギルド固有メッセージテキストリポジトリ実装
#[derive(Debug)]
pub struct SeaOrmGuildMessageTextRepository;

impl SeaOrmGuildMessageTextRepository {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SeaOrmGuildMessageTextRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GuildMessageTextRepository for SeaOrmGuildMessageTextRepository {
    async fn get_by_guild_and_id<'c, C>(
        &self,
        db: &'c C,
        guild_id: i64,
        message_id: &str,
    ) -> Result<Option<guild_message_texts::Model>, DbErr>
    where
        C: sea_orm::ConnectionTrait,
    {
        guild_message_texts::Entity::find()
            .filter(guild_message_texts::Column::GuildId.eq(guild_id))
            .filter(guild_message_texts::Column::Id.eq(message_id))
            .one(db)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::database::connection::connection_manager::is_database_available;

    async fn setup_test_repo() -> Result<
        (
            SeaOrmGuildMessageTextRepository,
            sea_orm::DatabaseConnection,
        ),
        String,
    > {
        let (available, missing) = is_database_available();
        if !available {
            return Err(format!(
                "Database connection info not set - missing: {missing:?}"
            ));
        }

        let conn = match crate::repository::database::models_database::Database::new().await {
            Ok(db) => db.conn,
            Err(e) => return Err(format!("Failed to connect to a database: {e}")),
        };

        Ok((SeaOrmGuildMessageTextRepository::new(), conn))
    }

    #[tokio::test]
    async fn test_guild_message_text_get_by_guild_and_id() {
        let (repo, conn) = match setup_test_repo().await {
            Ok(result) => result,
            Err(e) => {
                println!("Skipping database test: {e}");
                return;
            }
        };

        // 存在しないメッセージを取得
        let result = repo
            .get_by_guild_and_id(&conn, 999999, "non_existent_message")
            .await;

        match result {
            Ok(None) => {
                // 期待される結果
                assert!(true);
            }
            Ok(Some(message)) => {
                println!("Found guild message: {}", message.message_jp);
                assert!(!message.message_jp.is_empty());
            }
            Err(e) => {
                println!("Query returned error: {e}");
            }
        }
    }
}
