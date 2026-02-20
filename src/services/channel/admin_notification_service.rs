use crate::gateway::DiscordGateway;
use crate::models::entities::master::channel_types::GuildChannelType;
use crate::repository::GuildChannelRepository;
use crate::types::Result;
use crate::types::discord::{DiscordChannelId, MessageContent};
use sea_orm::DatabaseTransaction;
use std::sync::Arc;
use tracing::{info, warn};

/// 管理者通知サービス
///
/// bot実行中のエラーや設定不足を管理者通知チャンネル（channel_type=5）に送信する。
/// チャンネルが未設定の場合はwarnログを出力して処理を継続する。
pub struct AdminNotificationService<G, GCR>
where
    G: DiscordGateway,
    GCR: GuildChannelRepository,
{
    gateway: Arc<G>,
    guild_channel_repo: GCR,
}

impl<G, GCR> AdminNotificationService<G, GCR>
where
    G: DiscordGateway,
    GCR: GuildChannelRepository,
{
    pub fn new(gateway: Arc<G>, guild_channel_repo: GCR) -> Self {
        Self {
            gateway,
            guild_channel_repo,
        }
    }

    /// 管理者通知チャンネルにメッセージを送信
    ///
    /// # 引数
    /// - `txn`: データベーストランザクション
    /// - `guild_id`: ギルドID
    /// - `content`: 送信するメッセージ内容
    ///
    /// # 動作
    /// - 管理者通知チャンネル（channel_type=5）が設定されている場合は Discord に送信する
    /// - チャンネルが未設定の場合は warn ログを出力し `Ok(())` を返す
    ///   （通知失敗がメイン処理の失敗に波及しないようにする）
    pub async fn notify_admin(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        content: MessageContent,
    ) -> Result<()> {
        // 管理者通知チャンネルを取得
        let channel = self
            .guild_channel_repo
            .get_by_guild_and_type_with_txn(
                txn,
                guild_id,
                GuildChannelType::AdminNotification.as_i32(),
            )
            .await?;

        let Some(channel) = channel else {
            warn!(
                guild_id = guild_id,
                channel_type = GuildChannelType::AdminNotification.as_i32(),
                "管理者通知チャンネルが設定されていません。通知をスキップします"
            );
            return Ok(());
        };

        let channel_id = DiscordChannelId::new(channel.channel_id as u64);

        self.gateway.send_message(channel_id, content).await?;

        info!(
            guild_id = guild_id,
            channel_id = channel.channel_id,
            "管理者通知を送信しました"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::GatewayError;
    use crate::gateway::r#impl::mock_discord_gateway::MockDiscordGateway;
    use crate::infrastructure::database::connection::sea_orm_connection::DatabaseConnectionManager;
    use crate::models::entities::guild_master::guild_channels;
    use crate::models::entities::master::channel_types::GuildChannelType;
    use crate::types::discord::DiscordMessageId;
    use async_trait::async_trait;
    use chrono::Utc;
    use sea_orm::{DatabaseTransaction, TransactionTrait};

    /// テスト用GuildChannelRepositoryモック
    #[derive(Clone)]
    struct MockGuildChannelRepo {
        channel: Option<guild_channels::Model>,
    }

    #[async_trait]
    impl GuildChannelRepository for MockGuildChannelRepo {
        async fn upsert_with_txn(
            &self,
            _txn: &DatabaseTransaction,
            _guild_id: i64,
            _channel_type: i32,
            _channel_id: i64,
        ) -> crate::types::Result<guild_channels::Model> {
            unimplemented!()
        }

        async fn get_by_guild_and_type_with_txn(
            &self,
            _txn: &DatabaseTransaction,
            _guild_id: i64,
            _channel_type: i32,
        ) -> crate::types::Result<Option<guild_channels::Model>> {
            Ok(self.channel.clone())
        }

        async fn get_all_by_guild_with_txn(
            &self,
            _txn: &DatabaseTransaction,
            _guild_id: i64,
        ) -> crate::types::Result<Vec<guild_channels::Model>> {
            unimplemented!()
        }

        async fn delete_with_txn(
            &self,
            _txn: &DatabaseTransaction,
            _guild_id: i64,
            _channel_type: i32,
        ) -> crate::types::Result<()> {
            unimplemented!()
        }
    }

    /// テスト用トランザクションを作成するヘルパー
    async fn create_test_txn() -> Option<DatabaseTransaction> {
        let (available, _) = crate::test_utils::check_database_availability();
        if !available {
            return None;
        }

        let manager = DatabaseConnectionManager::new().await.ok()?;
        manager.connection().begin().await.ok()
    }

    /// テスト用guild_channelsモデルを作成するヘルパー
    fn make_channel_model(
        guild_id: i64,
        channel_type: i32,
        channel_id: i64,
    ) -> guild_channels::Model {
        guild_channels::Model {
            guild_id,
            channel_type,
            channel_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    // ============================================================
    // notify_admin
    // ============================================================

    /// 1-1: 正常系 - 管理者通知チャンネルが設定されている場合、メッセージを送信する
    #[tokio::test]
    async fn test_notify_admin_channel_exists() {
        let guild_id = 1001_i64;
        let channel_id = 9001_i64;
        let Some(txn) = create_test_txn().await else {
            println!("test_notify_admin_channel_exists: DB未設定のためスキップ");
            return;
        };

        // モック設定: チャンネルあり
        let repo = MockGuildChannelRepo {
            channel: Some(make_channel_model(
                guild_id,
                GuildChannelType::AdminNotification.as_i32(),
                channel_id,
            )),
        };

        // Discordゲートウェイモック: send_messageが1回呼ばれることを期待
        let mut mock_gateway = MockDiscordGateway::new();
        mock_gateway
            .expect_send_message()
            .times(1)
            .returning(|_, _| Ok(DiscordMessageId::new(12345)));

        let service = AdminNotificationService::new(Arc::new(mock_gateway), repo);

        let result = service
            .notify_admin(&txn, guild_id, MessageContent::text("エラーが発生しました"))
            .await;

        assert!(result.is_ok(), "通知送信に失敗: {:?}", result.err());
    }

    /// 1-2: 正常系 - 管理者通知チャンネルが未設定の場合、warnログを出しOkを返す
    #[tokio::test]
    async fn test_notify_admin_channel_not_configured() {
        let guild_id = 1002_i64;
        let Some(txn) = create_test_txn().await else {
            println!("test_notify_admin_channel_not_configured: DB未設定のためスキップ");
            return;
        };

        // モック設定: チャンネルなし
        let repo = MockGuildChannelRepo { channel: None };

        // Discordゲートウェイモック: send_messageは呼ばれないことを期待
        let mut mock_gateway = MockDiscordGateway::new();
        mock_gateway.expect_send_message().times(0);

        let service = AdminNotificationService::new(Arc::new(mock_gateway), repo);

        let result = service
            .notify_admin(&txn, guild_id, MessageContent::text("エラーが発生しました"))
            .await;

        // チャンネル未設定でもOkが返ること
        assert!(
            result.is_ok(),
            "チャンネル未設定時にOk以外が返った: {:?}",
            result.err()
        );
    }

    /// 1-3: 異常系 - Discord送信に失敗した場合、Errを返す
    #[tokio::test]
    async fn test_notify_admin_gateway_error() {
        let guild_id = 1003_i64;
        let channel_id = 9003_i64;
        let Some(txn) = create_test_txn().await else {
            println!("test_notify_admin_gateway_error: DB未設定のためスキップ");
            return;
        };

        // モック設定: チャンネルあり
        let repo = MockGuildChannelRepo {
            channel: Some(make_channel_model(
                guild_id,
                GuildChannelType::AdminNotification.as_i32(),
                channel_id,
            )),
        };

        // Discordゲートウェイモック: send_messageがエラーを返す
        let mut mock_gateway = MockDiscordGateway::new();
        mock_gateway
            .expect_send_message()
            .times(1)
            .returning(|_, _| {
                Err(GatewayError::SendMessageFailed(
                    "チャンネルが見つかりません".to_string(),
                ))
            });

        let service = AdminNotificationService::new(Arc::new(mock_gateway), repo);

        let result = service
            .notify_admin(&txn, guild_id, MessageContent::text("エラーが発生しました"))
            .await;

        assert!(result.is_err(), "送信失敗時にOkが返った");
    }
}
