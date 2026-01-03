use crate::models::guild_environments::GuildEnvironments;
use async_trait::async_trait;
use sea_orm::{DatabaseTransaction, DbErr};
use std::collections::HashMap;

/// ギルド環境設定リポジトリの抽象インターフェース
/// データベースアクセスの詳細を隠蔽し、ギルド単位の環境変数管理を提供
#[async_trait]
pub trait GuildEnvironmentRepository: Send + Sync {
    /// 特定のギルドの環境変数を取得
    async fn get_by_guild_and_key<'c, C>(
        &self,
        db: &'c C,
        guild_id: i64,
        key: &str,
    ) -> Result<Option<GuildEnvironments>, DbErr>
    where
        C: sea_orm::ConnectionTrait;

    /// 特定のギルドの複数環境変数を一括取得（パフォーマンス最適化）
    /// N+1問題を回避するため、複数のキーを一度に取得する
    async fn get_multiple_by_guild<'c, C>(
        &self,
        db: &'c C,
        guild_id: i64,
        keys: &[&str],
    ) -> Result<HashMap<String, String>, DbErr>
    where
        C: sea_orm::ConnectionTrait;

    /// トランザクション対応版 - 環境変数設定（Upsert: 存在しない場合は作成、存在する場合は更新）
    async fn set_with_txn(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        key: &str,
        value: &str,
    ) -> Result<GuildEnvironments, DbErr>;
}
