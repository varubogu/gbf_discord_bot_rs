use crate::repository::BattleRecruitmentRepository;
use crate::repository::database::battle_recruitment_repository::BattleRecruitmentRepositoryImpl;
use crate::types::PoiseError;
use async_trait::async_trait;
use sea_orm::{
    Database as SeaDatabase, DatabaseConnection, DatabaseTransaction as SeaOrmTransaction,
    TransactionTrait,
};
use std::env;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tracing::info;

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
        self.txn
            .as_ref()
            .ok_or_else(|| "Transaction already consumed".into())
    }
}

/// 自動ロールバック警告のためのDrop trait実装
///
/// トランザクションが明示的にコミットされずにドロップされた場合、
/// 警告ログを出力し、SeaORMによって自動的にロールバックが実行されます。
impl Drop for Transaction {
    fn drop(&mut self) {
        if !self.committed && self.txn.is_some() {
            tracing::warn!(
                "Transaction dropped without commit - rollback will occur automatically"
            );
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
    /// このメソッドは、ラムダ式を受け取ってトランザクションを提供します。
    /// **重要**: ラムダ式内で明示的に`txn.commit().await?`を呼び出してください。
    /// コミットが呼び出されずにラムダ式が終了した場合、トランザクションは自動的にロールバックされます。
    /// エラーが発生した場合も自動的にロールバックされます。
    ///
    /// # 型パラメータ
    /// * `F` - 実行するラムダ式の型
    /// * `T` - ラムダ式の戻り値の型
    ///
    /// # 引数
    /// * `f` - トランザクション内で実行するラムダ式（必ずコミットを呼び出すこと）
    ///
    /// # エラー
    /// トランザクション開始またはラムダ式の実行でエラーが発生した場合
    ///
    /// # 戻り値
    /// ラムダ式の実行結果、またはエラー
    ///
    /// # 例
    /// ```rust
    /// let result = db.execute_in_transaction(|txn| async move {
    ///     // データベース操作をここに記述
    ///     // 処理が成功したら必ずコミットを呼び出す
    ///     txn.commit().await?;
    ///     Ok(some_value)
    /// }).await?;
    /// ```
    async fn execute_in_transaction<F, T, Fut>(&self, f: F) -> Result<T, PoiseError>
    where
        F: FnOnce(Transaction) -> Fut + Send,
        Fut: Future<Output = Result<T, PoiseError>> + Send,
        T: Send,
    {
        let txn = self.begin_transaction().await?;
        let result = f(txn).await?;
        Ok(result)
    }
}

// すべてのDatabaseService実装に対してDatabaseServiceExtを自動実装
impl<T: ?Sized + DatabaseService> DatabaseServiceExt for T {}

/// 流暢なトランザクションAPIのためのトランザクションビルダー
///
/// このStructは、ラムダ式を受け取ってトランザクション内で実行する機能を提供します。
/// **重要**: ラムダ式内で明示的にコミットを呼び出してください。
/// コミットが呼び出されずにラムダ式が終了した場合、トランザクションは自動的にロールバックされます。
pub struct TransactionBuilder<'a> {
    /// データベースサービスへの参照
    db: &'a dyn DatabaseService,
}

impl<'a> TransactionBuilder<'a> {
    /// ラムダ式を受け取ってトランザクション内で実行
    ///
    /// このメソッドは自動的にトランザクションを開始し、提供されたラムダ式を実行します。
    /// **重要**: ラムダ式内で明示的に`txn.commit().await?`を呼び出してください。
    /// コミットが呼び出されずにラムダ式が終了した場合、トランザクションは自動的にロールバックされます。
    /// エラーが発生した場合も自動的にロールバックされます。
    ///
    /// # 型パラメータ
    /// * `F` - 実行するラムダ式の型
    /// * `T` - ラムダ式の戻り値の型
    ///
    /// # 引数
    /// * `f` - トランザクション内で実行するラムダ式（必ずコミットを呼び出すこと）
    ///
    /// # エラー
    /// トランザクション開始またはラムダ式の実行でエラーが発生した場合
    ///
    /// # 戻り値
    /// ラムダ式の実行結果、またはエラー
    ///
    /// # 例
    /// ```rust
    /// let result = db.transaction().execute(|txn| Box::pin(async move {
    ///     // データベース操作をここに記述
    ///     // 処理が成功したら必ずコミットを呼び出す
    ///     txn.commit().await?;
    ///     Ok(some_value)
    /// })).await?;
    /// ```
    pub async fn execute<F, T>(self, f: F) -> Result<T, PoiseError>
    where
        F: FnOnce(Transaction) -> Pin<Box<dyn Future<Output = Result<T, PoiseError>> + Send>>,
    {
        let txn = self.db.begin_transaction().await?;
        let result = f(txn).await?;
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

/// データベース接続マネージャー（Repository層専用）
pub struct DatabaseConnectionManager {
    conn: DatabaseConnection,
}

impl DatabaseConnectionManager {
    pub async fn new() -> Result<Self, sea_orm::DbErr> {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        info!("Connecting to database...");
        let conn = SeaDatabase::connect(&database_url).await?;

        info!("Connected to database");
        Ok(Self { conn })
    }

    pub fn connection(&self) -> &DatabaseConnection {
        &self.conn
    }
}

/// Repository層の依存注入コンテナ
pub struct RepositoryContainer {
    pub battle_recruitment_repo: Arc<dyn BattleRecruitmentRepository>,
    // 他のrepositoryも追加可能
}

impl RepositoryContainer {
    pub async fn new() -> Result<Self, PoiseError> {
        let db_manager = DatabaseConnectionManager::new().await?;

        let battle_recruitment_repo = Arc::new(BattleRecruitmentRepositoryImpl::new(
            db_manager.connection().clone(),
        ));

        Ok(Self {
            battle_recruitment_repo,
        })
    }
}

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
