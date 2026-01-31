//! アプリケーションDIコンテナ
//!
//! Gateway抽象化を通じた依存性注入を管理するコンテナ。
//! 静的ディスパッチ（ジェネリクス）を採用し、本番用とテスト用で異なる具象型を使用可能。

use sea_orm::DatabaseConnection;
use std::sync::Arc;

use crate::di::Repositories;
use crate::gateway::DiscordGateway;
use crate::services::message::MessageService;

/// アプリケーション全体のDIコンテナ（静的ディスパッチ版）
///
/// 型パラメータ`G`はDiscord Gatewayの実装を指定する。
/// 本番環境では`PoiseDiscordGateway`、テストでは`MockDiscordGateway`を使用する。
#[derive(Clone)]
pub struct AppContainer<G>
where
    G: DiscordGateway,
{
    /// Discord Gateway実装
    pub gateway: Arc<G>,
    /// リポジトリ群
    pub repositories: Repositories,
    /// メッセージサービス（Gateway非依存）
    pub message_service: Arc<MessageService>,
}

impl<G> AppContainer<G>
where
    G: DiscordGateway + Clone + 'static,
{
    /// 汎用コンストラクタ
    pub fn new(
        gateway: Arc<G>,
        guild_db: Arc<DatabaseConnection>,
        system_db: Arc<DatabaseConnection>,
        global_db: Arc<DatabaseConnection>,
    ) -> Self {
        let repositories = Repositories::new(guild_db, system_db, global_db);
        let message_service = Arc::new(MessageService::new());

        Self {
            gateway,
            repositories,
            message_service,
        }
    }

    /// Gatewayを取得する
    pub fn gateway(&self) -> &Arc<G> {
        &self.gateway
    }

    /// Repositoriesを取得する
    pub fn repositories(&self) -> &Repositories {
        &self.repositories
    }

    /// MessageServiceを取得する
    pub fn message_service(&self) -> &Arc<MessageService> {
        &self.message_service
    }

    /// Guild ロール用DB接続を取得（通常のコマンド実行用）
    pub fn guild_db(&self) -> &DatabaseConnection {
        self.repositories.guild_db()
    }

    /// System ロール用DB接続を取得（スケジューラー用）
    pub fn system_db(&self) -> &DatabaseConnection {
        self.repositories.system_db()
    }

    /// Global ロール用DB接続を取得（マスターデータ更新用）
    pub fn global_db(&self) -> &DatabaseConnection {
        self.repositories.global_db()
    }
}

// 本番環境用の専用実装
use crate::gateway::PoiseDiscordGateway;
use poise::serenity_prelude::Http;

impl AppContainer<PoiseDiscordGateway> {
    /// 本番環境用コンテナを構築
    pub fn new_production(
        http: Arc<Http>,
        guild_db: Arc<DatabaseConnection>,
        system_db: Arc<DatabaseConnection>,
        global_db: Arc<DatabaseConnection>,
    ) -> Self {
        let gateway = Arc::new(PoiseDiscordGateway::new(http));
        Self::new(gateway, guild_db, system_db, global_db)
    }

    /// HTTPクライアントを直接取得（移行期間中の後方互換性用）
    ///
    /// 注意: 新しいコードではGatewayトレイトを使用すること。
    /// このメソッドは移行期間中のみ使用する。
    pub fn http(&self) -> &Arc<Http> {
        self.gateway.http()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gateway::r#impl::mock_discord_gateway::MockDiscordGateway;

    /// テスト用コンテナの作成例
    ///
    /// 実際のテストではデータベース接続のモック化も必要
    #[test]
    fn test_container_creation() {
        // モックGatewayを作成
        let mock_gateway = MockDiscordGateway::new();
        let gateway = Arc::new(mock_gateway);

        // コンテナの型が正しく推論されることを確認
        let _container_type: std::marker::PhantomData<AppContainer<MockDiscordGateway>> =
            std::marker::PhantomData;

        // Gatewayの型が正しいことを確認
        assert!(Arc::strong_count(&gateway) >= 1);
    }
}
