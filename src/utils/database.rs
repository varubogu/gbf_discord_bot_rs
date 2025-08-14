use std::pin::Pin;
use std::future::Future;
use async_trait::async_trait;
use sea_orm::{DatabaseConnection, DatabaseTransaction as SeaOrmTransaction, TransactionTrait};
use crate::types::PoiseError;

/// データベーストランザクションを管理するジェネリックラッパー
/// 
/// このStructはSeaORMトランザクションをラップし、適切なコミット/ロールバック動作を保証します。
/// トランザクションが明示的にコミットされずにドロップされた場合、自動的にロールバックされます。
/// 
/// # 例
/// ```rust
/// let txn = db.begin_transaction().await?;
/// // データベース操作を実行
/// txn.commit().await?; // 明示的なコミット
/// ```
pub struct Transaction {
    /// 内部のSeaORMトランザクション
    txn: Option<SeaOrmTransaction>,
    /// コミット済みフラグ
    committed: bool,
}

impl Transaction {
    /// 新しいTransactionインスタンスを作成
    /// 
    /// # 引数
    /// * `txn` - SeaORMのデータベーストランザクション
    /// 
    /// # 戻り値
    /// 新しいTransactionインスタンス
    pub fn new(txn: SeaOrmTransaction) -> Self {
        Self {
            txn: Some(txn),
            committed: false,
        }
    }

    /// トランザクションをコミットする
    /// 
    /// このメソッドを呼び出すとトランザクションの全ての変更がデータベースに永続化されます。
    /// コミット後、このTransactionインスタンスは使用できなくなります。
    /// 
    /// # エラー
    /// データベースエラーが発生した場合、`sea_orm::DbErr`を返します。
    /// 
    /// # 例
    /// ```rust
    /// let txn = db.begin_transaction().await?;
    /// // データベース操作を実行
    /// txn.commit().await?;
    /// ```
    pub async fn commit(mut self) -> Result<(), sea_orm::DbErr> {
        if let Some(txn) = self.txn.take() {
            txn.commit().await?;
            self.committed = true;
        }
        Ok(())
    }

    /// リポジトリで使用するための内部SeaORMトランザクションの参照を取得
    /// 
    /// # エラー
    /// トランザクションが既に消費されている場合、エラーを返します。
    /// 
    /// # 戻り値
    /// SeaORMトランザクションへの参照、またはエラー
    pub fn get_txn(&self) -> Result<&SeaOrmTransaction, PoiseError> {
        self.txn.as_ref().ok_or_else(|| "Transaction already consumed".into())
    }
}

/// 自動ロールバック警告のためのDrop trait実装
/// 
/// トランザクションが明示的にコミットされずにドロップされた場合、
/// 警告ログを出力し、SeaORMによって自動的にロールバックが実行されます。
impl Drop for Transaction {
    fn drop(&mut self) {
        if !self.committed && self.txn.is_some() {
            tracing::warn!("Transaction dropped without commit - rollback will occur automatically");
        }
    }
}

/// トランザクション管理のための汎用データベースサービストレイト
/// 
/// このトレイトは、データベース接続の抽象化とトランザクション管理機能を提供します。
/// 具体的な実装（SeaOrmDatabase等）は、このトレイトを実装する必要があります。
#[async_trait]
pub trait DatabaseService: Send + Sync + std::fmt::Debug {
    /// 新しいトランザクションを開始
    /// 
    /// # エラー
    /// データベース接続エラーやトランザクション開始に失敗した場合、エラーを返します。
    /// 
    /// # 戻り値
    /// 新しいTransactionインスタンス、またはエラー
    async fn begin_transaction(&self) -> Result<Transaction, PoiseError>;

    /// 複雑な操作のためのトランザクションビルダーを作成
    /// 
    /// このメソッドは流暢なAPIスタイルでトランザクションを管理するためのビルダーを返します。
    /// ラムダ式を受け取り、その中で自動的にトランザクションが管理されます。
    fn transaction(&self) -> TransactionBuilder<'_>
    where
        Self: Sized,
    {
        TransactionBuilder { db: self }
    }

    /// リポジトリで使用するための基底データベース接続を取得
    /// 
    /// # 戻り値
    /// DatabaseConnectionへの参照
    fn get_connection(&self) -> &DatabaseConnection;
}

/// DatabaseServiceの拡張トレイト（ラムダスタイルトランザクション用）
/// 
/// このトレイトは、コンクリート型でのみ利用可能なラムダスタイルトランザクション機能を提供します。
/// トレイトオブジェクトでは使用できませんが、型安全性とパフォーマンスを提供します。
pub trait DatabaseServiceExt: DatabaseService {
    /// ラムダ式を使用してトランザクション内で操作を実行
    /// 
    /// このメソッドは、ラムダ式を受け取って自動的にトランザクションを管理します。
    /// エラーが発生した場合は自動的にロールバックされます。
    /// 
    /// # 型パラメータ
    /// * `F` - 実行するラムダ式の型
    /// * `T` - ラムダ式の戻り値の型
    /// 
    /// # 引数
    /// * `f` - トランザクション内で実行するラムダ式
    /// 
    /// # エラー
    /// トランザクション開始、ラムダ式の実行、またはコミットでエラーが発生した場合
    /// 
    /// # 戻り値
    /// ラムダ式の実行結果、またはエラー
    async fn execute_in_transaction<F, T, Fut>(&self, f: F) -> Result<T, PoiseError>
    where
        F: FnOnce(&Transaction) -> Fut + Send,
        Fut: Future<Output = Result<T, PoiseError>> + Send,
        T: Send,
    {
        let txn = self.begin_transaction().await?;
        let result = f(&txn).await?;
        txn.commit().await.map_err(|e| Box::new(e) as PoiseError)?;
        Ok(result)
    }
}

// すべてのDatabaseService実装に対してDatabaseServiceExtを自動実装
impl<T: ?Sized + DatabaseService> DatabaseServiceExt for T {}

/// 流暢なトランザクションAPIのためのトランザクションビルダー
/// 
/// このStructは、ラムダ式を受け取ってトランザクション内で実行し、
/// 自動的にコミット・ロールバック処理を管理する機能を提供します。
pub struct TransactionBuilder<'a> {
    /// データベースサービスへの参照
    db: &'a dyn DatabaseService,
}

impl<'a> TransactionBuilder<'a> {
    /// ラムダ式を受け取ってトランザクション内で実行
    /// 
    /// このメソッドは自動的にトランザクションを開始し、提供されたラムダ式を実行します。
    /// ラムダ式が成功した場合はトランザクションをコミットし、
    /// エラーが発生した場合は自動的にロールバックされます。
    /// 
    /// # 型パラメータ
    /// * `F` - 実行するラムダ式の型
    /// * `T` - ラムダ式の戻り値の型
    /// 
    /// # 引数
    /// * `f` - トランザクション内で実行するラムダ式
    /// 
    /// # エラー
    /// トランザクション開始、ラムダ式の実行、またはコミットでエラーが発生した場合
    /// 
    /// # 戻り値
    /// ラムダ式の実行結果、またはエラー
    /// 
    /// # 例
    /// ```rust
    /// let result = db.transaction().execute(|txn| Box::pin(async move {
    ///     // データベース操作をここに記述
    ///     Ok(some_value)
    /// })).await?;
    /// ```
    pub async fn execute<F, T>(self, f: F) -> Result<T, PoiseError>
    where
        F: FnOnce(&Transaction) -> Pin<Box<dyn Future<Output = Result<T, PoiseError>> + Send + '_>>,
    {
        let txn = self.db.begin_transaction().await?;
        let result = f(&txn).await?;
        txn.commit().await.map_err(|e| Box::new(e) as PoiseError)?;
        Ok(result)
    }
}

/// DatabaseServiceのSeaORM実装
/// 
/// このStructは、SeaORMを使用したデータベース接続とトランザクション管理を提供します。
/// PoiseDataで保持され、Facadeでトランザクション処理に使用されることを想定しています。
#[derive(Debug)]
pub struct SeaOrmDatabase {
    /// SeaORMデータベース接続
    conn: DatabaseConnection,
}

impl SeaOrmDatabase {
    /// 新しいSeaOrmDatabaseインスタンスを作成
    /// 
    /// # 引数
    /// * `conn` - SeaORMデータベース接続
    /// 
    /// # 戻り値
    /// 新しいSeaOrmDatabaseインスタンス
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl DatabaseService for SeaOrmDatabase {
    /// SeaORMを使用して新しいトランザクションを開始
    /// 
    /// このメソッドはSeaORMのデータベース接続からトランザクションを開始し、
    /// Transactionラッパーでラップして返します。
    /// 
    /// # エラー
    /// データベース接続エラーやトランザクション開始に失敗した場合、エラーを返します。
    /// 
    /// # 戻り値
    /// 新しいTransactionインスタンス、またはエラー
    async fn begin_transaction(&self) -> Result<Transaction, PoiseError> {
        let txn = self.conn.begin().await?;
        Ok(Transaction::new(txn))
    }

    /// 基底のSeaORMデータベース接続を取得
    /// 
    /// リポジトリパターンで直接データベース接続が必要な場合に使用します。
    /// トランザクション外での単純なクエリに適用されます。
    /// 
    /// # 戻り値
    /// SeaORMのDatabaseConnectionへの参照
    fn get_connection(&self) -> &DatabaseConnection {
        &self.conn
    }
}
