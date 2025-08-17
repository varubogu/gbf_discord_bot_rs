use crate::infrastructure::database::connection::{DatabaseConnectionManager, SeaOrmDatabase};
use crate::infrastructure::database::container::RepositoryContainer;
use crate::infrastructure::database::transaction::{DatabaseServiceExt, Transaction};
use crate::types::PoiseError;
use std::future::Future;
use std::pin::Pin;

/// トランザクションコンテキスト（Repository層のトランザクション対応メソッド用）
pub struct TransactionContext<'a> {
    pub txn: &'a Transaction,
    pub repos: &'a RepositoryContainer,
}

impl<'a> TransactionContext<'a> {
    pub fn new(txn: &'a Transaction, repos: &'a RepositoryContainer) -> Self {
        Self { txn, repos }
    }
}

/// トランザクション実行のための抽象化インターフェース
pub struct TransactionManager {
    db_service: SeaOrmDatabase,
    repos: RepositoryContainer,
}

impl TransactionManager {
    pub async fn new() -> Result<Self, PoiseError> {
        let db_manager = DatabaseConnectionManager::new().await?;
        let db_service = SeaOrmDatabase::new(db_manager.connection().clone());
        let repos = RepositoryContainer::new().await?;

        Ok(Self { db_service, repos })
    }

    /// Facade専用：トランザクション内で処理を実行
    pub async fn execute_in_transaction<F, T>(&self, f: F) -> Result<T, PoiseError>
    where
        F: FnOnce(
                TransactionContext,
            ) -> Pin<Box<dyn Future<Output = Result<T, PoiseError>> + Send>>
            + Send,
        T: Send,
    {
        self.db_service
            .execute_in_transaction(|txn| {
                let ctx = TransactionContext::new(&txn, &self.repos);
                f(ctx)
            })
            .await
    }
}
