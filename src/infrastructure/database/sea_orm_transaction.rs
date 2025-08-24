use crate::types::AppError;
use crate::types::transaction::{DatabaseConnectionTrait, DatabaseTransactionTrait};
use async_trait::async_trait;
use sea_orm::{DatabaseConnection, DatabaseTransaction, TransactionTrait};

/// SeaORM用トランザクション実装
pub struct SeaOrmTransaction {
    inner: DatabaseTransaction,
}

impl SeaOrmTransaction {
    pub fn new(inner: DatabaseTransaction) -> Self {
        Self { inner }
    }

    /// 内部のSeaOrmトランザクションへの参照を取得（Repository層で使用）
    pub fn inner(&self) -> &DatabaseTransaction {
        &self.inner
    }
}

#[async_trait]
impl DatabaseTransactionTrait for SeaOrmTransaction {
    type Error = sea_orm::DbErr;

    async fn commit(self) -> Result<(), Self::Error> {
        self.inner.commit().await
    }

    async fn rollback(self) -> Result<(), Self::Error> {
        self.inner.rollback().await
    }
}

/// SeaORM用接続実装
pub struct SeaOrmConnection {
    inner: DatabaseConnection,
}

impl SeaOrmConnection {
    pub fn new(inner: DatabaseConnection) -> Self {
        Self { inner }
    }

    /// 内部のSeaOrm接続への参照を取得
    pub fn inner(&self) -> &DatabaseConnection {
        &self.inner
    }
}

#[async_trait]
impl DatabaseConnectionTrait for SeaOrmConnection {
    type Transaction = SeaOrmTransaction;
    type Error = sea_orm::DbErr;

    async fn begin_transaction(&self) -> Result<Self::Transaction, Self::Error> {
        let txn = self.inner.begin().await?;
        Ok(SeaOrmTransaction::new(txn))
    }
}

/// SeaOrmのDbErrからAppErrorへの変換
impl From<sea_orm::DbErr> for AppError {
    fn from(err: sea_orm::DbErr) -> Self {
        AppError::Database(err)
    }
}
