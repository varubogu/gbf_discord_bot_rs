use async_trait::async_trait;
use std::future::Future;
use std::pin::Pin;

/// データベース非依存のトランザクション抽象化
#[async_trait]
pub trait DatabaseTransactionTrait: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    /// トランザクションをコミットする
    async fn commit(self) -> Result<(), Self::Error>;

    /// トランザクションをロールバックする
    async fn rollback(self) -> Result<(), Self::Error>;
}

/// データベース接続の抽象化
#[async_trait]
pub trait DatabaseConnectionTrait: Send + Sync {
    type Transaction: DatabaseTransactionTrait;
    type Error: std::error::Error + Send + Sync + 'static;

    /// トランザクションを開始する
    async fn begin_transaction(&self) -> Result<Self::Transaction, Self::Error>;
}

// /// トランザクション実行を抽象化するトレイト
// #[async_trait]
// pub trait TransactionManagerTrait: Send + Sync {
//     type Error: std::error::Error + Send + Sync + 'static;

//     /// トランザクション内で処理を実行する
//     async fn execute_in_transaction<F, T>(&self, f: F) -> Result<T, Self::Error>
//     where
//         F: FnOnce() -> Pin<Box<dyn Future<Output = Result<T, Self::Error>> + Send>> + Send,
//         T: Send;
// }
