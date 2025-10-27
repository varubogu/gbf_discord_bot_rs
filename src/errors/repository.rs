/// Repository層エラー
///
/// データアクセス関連のエラーを定義します。
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("データが見つかりません: {entity_type} (ID: {id})")]
    NotFound { entity_type: String, id: String },

    #[error("データベース接続エラー")]
    ConnectionError {
        #[from]
        source: sea_orm::DbErr,
    },

    #[error("トランザクションエラー: {message}")]
    TransactionError { message: String },

    #[error("制約違反エラー: {constraint}")]
    ConstraintViolation { constraint: String },

    #[error("データベースクエリエラー: {query}")]
    QueryError {
        query: String,
        #[source]
        source: sea_orm::DbErr,
    },
}
