pub mod battle_recruitment_repository;
pub mod quest_repository;
pub mod message_text_repository;
pub mod environment_repository;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::{DatabaseConnection, DatabaseTransaction as SeaOrmTransaction, TransactionTrait};
use tracing::info;
use crate::types::PoiseError;
use crate::utils::database::Transaction;
use crate::models::battle_recruitment::BattleRecruitment;
use crate::models::quest::{Quest, QuestAlias};
use crate::models::message_text::MessageText;
use crate::models::environment::Environment;
use crate::repository::{BattleRecruitmentRepository, QuestRepository, MessageTextRepository, EnvironmentRepository};

// Import repository implementations only (traits are imported from crate::repository)
use battle_recruitment_repository::SeaOrmBattleRecruitmentRepository;
use quest_repository::SeaOrmQuestRepository;
use message_text_repository::SeaOrmMessageTextRepository;
use environment_repository::SeaOrmEnvironmentRepository;

/// データベースプロバイダー - SeaORM実装の詳細を隠蔽
pub struct DatabaseProvider {
    conn: DatabaseConnection,
}

impl DatabaseProvider {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }

    pub fn get_connection(&self) -> &DatabaseConnection {
        &self.conn
    }

    pub async fn begin_transaction(&self) -> Result<Transaction, PoiseError> {
        let txn = self.conn.begin().await?;
        Ok(Transaction::new(txn))
    }
}

/// 抽象化されたトランザクション処理
pub struct DatabaseTransactionManager {
    provider: DatabaseProvider,
}

impl DatabaseTransactionManager {
    pub fn new(provider: DatabaseProvider) -> Self {
        Self { provider }
    }

    pub async fn execute_with_transaction<F, R, Fut>(&self, operation: F) -> Result<R, PoiseError>
    where
        F: FnOnce(&DatabaseTransactionContext) -> Fut + Send,
        Fut: std::future::Future<Output = Result<R, PoiseError>> + Send,
        R: Send,
    {
        let txn = self.provider.begin_transaction().await?;
        let ctx = DatabaseTransactionContext { txn: &txn };
        let result = operation(&ctx).await;
        
        match result {
            Ok(value) => {
                txn.commit().await?;
                Ok(value)
            }
            Err(e) => {
                // トランザクションは自動的にロールバックされる
                Err(e)
            }
        }
    }
}

/// トランザクションコンテキスト（実装の詳細を隠蔽）
pub struct DatabaseTransactionContext<'a> {
    txn: &'a Transaction,
}

impl<'a> DatabaseTransactionContext<'a> {
    pub fn get_transaction(&self) -> &Transaction {
        self.txn
    }
}

/// 抽象化されたリポジトリ実装 - バトル募集
pub struct BattleRecruitmentRepositoryImpl {
    inner: SeaOrmBattleRecruitmentRepository,
}

impl BattleRecruitmentRepositoryImpl {
    pub fn new(provider: DatabaseProvider) -> Self {
        let inner = SeaOrmBattleRecruitmentRepository::new(provider.get_connection().clone());
        Self { inner }
    }
}

#[async_trait]
impl crate::repository::BattleRecruitmentRepository for BattleRecruitmentRepositoryImpl {
    async fn create(&self, 
        guild_id: i64,
        channel_id: i64,
        message_id: i64,
        target_id: i32,
        battle_type_id: i32,
        expiry_date: DateTime<Utc>,
    ) -> Result<BattleRecruitment, PoiseError> {
        self.inner.create(guild_id, channel_id, message_id, target_id, battle_type_id, expiry_date).await
    }

    async fn get_by_message(&self, 
        guild_id: i64, 
        channel_id: i64, 
        message_id: i64
    ) -> Result<Option<BattleRecruitment>, PoiseError> {
        self.inner.get_by_message(guild_id, channel_id, message_id).await
    }

    async fn set_end_message(&self, 
        recruitment_id: i32, 
        message_id: i64
    ) -> Result<(), PoiseError> {
        self.inner.set_end_message(recruitment_id, message_id).await
    }
}

/// 抽象化されたリポジトリ実装 - クエスト
pub struct QuestRepositoryImpl {
    inner: SeaOrmQuestRepository,
}

impl QuestRepositoryImpl {
    pub fn new(provider: DatabaseProvider) -> Self {
        let inner = SeaOrmQuestRepository::new(provider.get_connection().clone());
        Self { inner }
    }
}

#[async_trait]
impl crate::repository::QuestRepository for QuestRepositoryImpl {
    async fn get_all(&self) -> Result<Vec<Quest>, PoiseError> {
        self.inner.get_all().await
    }
    
    async fn get_aliases(&self) -> Result<Vec<QuestAlias>, PoiseError> {
        self.inner.get_aliases().await
    }
    
    async fn get_by_alias(&self, alias: &str) -> Result<Option<Quest>, PoiseError> {
        self.inner.get_by_alias(alias).await
    }
    
    async fn get_by_target_id(&self, target_id: i32) -> Result<Option<Quest>, PoiseError> {
        self.inner.get_by_target_id(target_id).await
    }
}

/// 抽象化されたリポジトリ実装 - メッセージテキスト
pub struct MessageTextRepositoryImpl {
    inner: SeaOrmMessageTextRepository,
}

impl MessageTextRepositoryImpl {
    pub fn new(provider: DatabaseProvider) -> Self {
        let inner = SeaOrmMessageTextRepository::new(provider.get_connection().clone());
        Self { inner }
    }
}

#[async_trait]
impl crate::repository::MessageTextRepository for MessageTextRepositoryImpl {
    async fn get_by_guild_and_message(&self, guild_id: i64, message_id: &str) -> Result<Option<MessageText>, PoiseError> {
        self.inner.get_by_guild_and_message(guild_id, message_id).await
    }
}

/// 抽象化されたリポジトリ実装 - 環境設定
pub struct EnvironmentRepositoryImpl {
    inner: SeaOrmEnvironmentRepository,
}

impl EnvironmentRepositoryImpl {
    pub fn new(provider: DatabaseProvider) -> Self {
        let inner = SeaOrmEnvironmentRepository::new(provider.get_connection().clone());
        Self { inner }
    }
}

#[async_trait]
impl crate::repository::EnvironmentRepository for EnvironmentRepositoryImpl {
    async fn get_all(&self) -> Result<Vec<Environment>, PoiseError> {
        self.inner.get_all().await
    }
    
    async fn get_by_key(&self, key: &str) -> Result<Option<Environment>, PoiseError> {
        self.inner.get_by_key(key).await
    }
    
    async fn set(&self, key: &str, value: &str) -> Result<Environment, PoiseError> {
        self.inner.set(key, value).await
    }
}

pub struct Database {
    pub quest: Box<dyn QuestRepository + Send + Sync>,
    pub battle_recruitment: Box<dyn BattleRecruitmentRepository + Send + Sync>,
    pub message_text: Box<dyn MessageTextRepository + Send + Sync>,
    pub environment: Box<dyn EnvironmentRepository + Send + Sync>,
}

impl Database {
    pub async fn new() -> Result<Self, sqlx::Error> {
        info!("Creating database connection...");
        
        // Get database connection
        let conn = match crate::models::database::Database::new().await {
            Ok(db) => db.conn,
            Err(e) => return Err(sqlx::Error::Protocol(format!("Failed to connect to database: {}", e))),
        };

        info!("Connected to database");
        Ok(Self {
            quest: Box::new(SeaOrmQuestRepository::new(conn.clone())),
            battle_recruitment: Box::new(SeaOrmBattleRecruitmentRepository::new(conn.clone())),
            message_text: Box::new(SeaOrmMessageTextRepository::new(conn.clone())),
            environment: Box::new(SeaOrmEnvironmentRepository::new(conn)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn test_database_new() {
        // Test database creation
        // Note: This test will be skipped if DATABASE_URL is not set
        if std::env::var("DATABASE_URL").is_err() {
            println!("Skipping database test: DATABASE_URL not set");
            return;
        }

        let result = Database::new().await;
        assert!(result.is_ok(), "Database creation should succeed with valid connection");
    }

}