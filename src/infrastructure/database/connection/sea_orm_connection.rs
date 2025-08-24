use crate::infrastructure::database::connection::build_database_url;
use crate::types::Result;
use sea_orm::{Database as SeaDatabase, DatabaseConnection, TransactionTrait};
use std::env;
use tracing::info;

/// DatabaseServiceのSeaORM実装
///
/// このStructは、SeaORMを使用したデータベース接続を提供します。
/// AppStateで保持され、Facadeでトランザクション処理に使用されることを想定しています。
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

    /// SeaORMのDatabaseConnectionを取得
    ///
    /// AppStateパターンで直接データベース接続が必要な場合に使用します。
    ///
    /// # 戻り値
    /// SeaORMのDatabaseConnectionへの参照
    pub fn get_connection(&self) -> &DatabaseConnection {
        &self.conn
    }
}

/// データベース接続マネージャー（Repository層専用）
pub struct DatabaseConnectionManager {
    conn: DatabaseConnection,
}

impl DatabaseConnectionManager {
    pub async fn new() -> std::result::Result<Self, sea_orm::DbErr> {
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
