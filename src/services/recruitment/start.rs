use sea_orm::DatabaseTransaction;
use tracing::{info, warn};

use crate::models::battle_recruitments::BattleRecruitments;
use crate::repository::battle_recruitments_repository::BattleRecruitmentsRepository;
use crate::types::discord::{DiscordChannelId, DiscordGuildId, DiscordMessageId};
use crate::types::{AppError, Result};

/// StartRecruitmentService - 募集開始処理を行うサービス
pub struct StartRecruitmentService<R: BattleRecruitmentsRepository> {
    repo: R,
}

impl<R: BattleRecruitmentsRepository> StartRecruitmentService<R> {
    /// 新しいStartRecruitmentServiceを作成（依存性注入）
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    /// メッセージIDから募集を開始する（Facade層用メソッド）
    pub async fn start_by_message(
        &self,
        guild_id: u64,
        channel_id: u64,
        message_id: u64,
        txn: &DatabaseTransaction,
    ) -> Result<BattleRecruitments> {
        info!("StartRecruitmentService::start_by_message - 開始処理開始");

        // 募集情報の存在確認
        let recruitment = self
            .get_recruitment_from_db(txn, guild_id, channel_id, message_id)
            .await?;

        // 募集を開始済み状態に更新（message_idを使用）
        self.mark_recruitment_as_started(txn, recruitment.id as i64, message_id)
            .await?;

        info!(recruitment_id = recruitment.id, "開始処理完了");
        Ok(recruitment)
    }

    /// DBから募集情報を取得（トランザクション内で実行）
    pub async fn get_recruitment_from_db(
        &self,
        txn: &sea_orm::DatabaseTransaction,
        guild_id: u64,
        channel_id: u64,
        message_id: u64,
    ) -> Result<BattleRecruitments> {
        info!(
            "DB募集情報取得開始: guild_id={}, channel_id={}, message_id={}",
            guild_id, channel_id, message_id
        );

        // u64をドメイン型に変換
        match self
            .repo
            .get_by_message_with_txn(
                txn,
                DiscordGuildId::new(guild_id),
                DiscordChannelId::new(channel_id),
                DiscordMessageId::new(message_id),
            )
            .await?
        {
            Some(recruitment) => {
                info!("募集情報取得成功: recruitment_id={}", recruitment.id);
                Ok(recruitment)
            }
            None => {
                warn!("募集情報が見つかりません: message_id={}", message_id);
                Err(AppError::NotFound(format!(
                    "Recruitment not found for message_id: {message_id}"
                )))
            }
        }
    }

    /// 開始メッセージを作成（参加者へのメンション含む）
    pub async fn create_start_message(
        &self,
        quest_name: &str,
        participants: &[String],
    ) -> Result<String> {
        let participant_mentions = if participants.is_empty() {
            "参加者がいません".to_string()
        } else {
            participants.join(" ")
        };

        let message = format!(
            "🚀 **クエスト出発時間です！** 🚀\n\n{quest_name}\n\n参加者の皆さん: {participant_mentions}\n\nクエストを開始してください！"
        );
        Ok(message)
    }

    /// 募集を開始済み状態に更新（トランザクション内で実行）
    /// 注意: 現在のBattleRecruitmentRepositoryトレイトには開始済み状態更新メソッドがないため、
    /// set_end_messageを使用して終了メッセージIDを設定することで開始状態を表現します。
    pub async fn mark_recruitment_as_started(
        &self,
        txn: &sea_orm::DatabaseTransaction,
        recruitment_id: i64,
        end_message_id: u64,
    ) -> Result<()> {
        info!(
            "募集開始済み状態更新開始: recruitment_id={}, end_message_id={}",
            recruitment_id, end_message_id
        );

        self.repo
            .set_end_message_with_txn(
                txn,
                recruitment_id as i32,
                DiscordMessageId::new(end_message_id),
            )
            .await?;

        info!(
            "募集開始済み状態更新成功: recruitment_id={}",
            recruitment_id
        );
        Ok(())
    }
}
