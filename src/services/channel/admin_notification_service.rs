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
