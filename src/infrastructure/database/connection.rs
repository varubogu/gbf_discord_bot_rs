use crate::infrastructure::database::transaction::{DatabaseService, Transaction};
use crate::services::database::connection::build_database_url;
use crate::types::PoiseError;
use async_trait::async_trait;
use sea_orm::{Database as SeaDatabase, DatabaseConnection, TransactionTrait};
use std::env;
use tracing::info;

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
        // 集約されたdatabase connectionサービスを使用してURLを構築
        let database_url = build_database_url().map_err(|e| sea_orm::DbErr::Custom(e.message))?;

        info!("Connecting to database...");
        let conn = SeaDatabase::connect(&database_url).await?;

        info!("Connected to database");
        Ok(Self { conn })
    }

    pub fn connection(&self) -> &DatabaseConnection {
        &self.conn
    }
}
