use crate::infrastructure::database::container::RepositoryContainer;
use crate::infrastructure::database::sea_orm_transaction::SeaOrmTransaction;
use crate::types::Result;
use crate::types::transaction::DatabaseTransactionTrait;
use sea_orm::{DatabaseConnection, TransactionTrait};
use std::future::Future;
use std::sync::Arc;

/// SeaORM用のトランザクションコンテキスト（実用的な設計）
pub struct TransactionContext<'a> {
    pub txn: &'a SeaOrmTransaction,
    pub repos: &'a RepositoryContainer,
}

impl<'a> TransactionContext<'a> {
    pub fn new(txn: &'a SeaOrmTransaction, repos: &'a RepositoryContainer) -> Self {
        Self { txn, repos }
    }

    /// Repository層が使用するSeaORMトランザクションへのアクセス
    pub fn sea_orm_txn(&self) -> &sea_orm::DatabaseTransaction {
        self.txn.inner()
    }
}

/// トランザクション実行のための実用的なTransactionManager
pub struct TransactionManager {
    db_connection: Arc<DatabaseConnection>,
    repos: RepositoryContainer,
}

impl TransactionManager {
    /// 依存性注入対応のコンストラクタ（推奨）
    pub fn new(db_connection: Arc<DatabaseConnection>) -> Self {
        let repos = RepositoryContainer::new(&db_connection);
        Self {
            db_connection,
            repos,
        }
    }

    /// AppStateから作成するファクトリメソッド（Guildロールを使用）
    pub fn from_app_state(app_state: &crate::types::AppState) -> Self {
        Self::new(app_state.guild_db.clone())
    }

    /// Facade専用：トランザクション内で処理を実行
    pub async fn execute_in_transaction<F, Fut, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(TransactionContext) -> Fut,
        Fut: Future<Output = Result<T>> + Send,
        T: Send,
    {
        let sea_orm_txn = self.db_connection.begin().await?;
        let wrapped_txn = SeaOrmTransaction::new(sea_orm_txn);

        let result = {
            let ctx = TransactionContext::new(&wrapped_txn, &self.repos);
            f(ctx).await
        };

        match result {
            Ok(value) => {
                wrapped_txn.commit().await?;
                Ok(value)
            }
            Err(e) => {
                let _ = wrapped_txn.rollback().await; // ロールバックエラーは無視
                Err(e)
            }
        }
    }
}
