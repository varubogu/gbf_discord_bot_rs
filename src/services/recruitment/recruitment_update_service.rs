use crate::repository::BattleRecruitmentsRepository;
use crate::types::Result;
use crate::types::discord::DiscordMessageId;
use chrono::{DateTime, Utc};
use sea_orm::DatabaseTransaction;
use tracing::info;

/// 募集情報更新Service
/// 募集情報の更新操作の責務を持つ
pub struct RecruitmentUpdateService<BR: BattleRecruitmentsRepository> {
    battle_recruitment_repo: BR,
}

impl<BR: BattleRecruitmentsRepository> RecruitmentUpdateService<BR> {
    pub fn new(battle_recruitment_repo: BR) -> Self {
        Self {
            battle_recruitment_repo,
        }
    }

    /// 募集情報を更新
    pub async fn update_recruitment(
        &self,
        txn: &DatabaseTransaction,
        recruitment_id: i32,
        quest_id: i32,
        battle_style_id: i32,
        expiry_date: DateTime<Utc>,
    ) -> Result<()> {
        self.battle_recruitment_repo
            .update_with_txn(txn, recruitment_id, quest_id, battle_style_id, expiry_date)
            .await?;

        info!(
            recruitment_id = recruitment_id,
            quest_id = quest_id,
            battle_style_id = battle_style_id,
            "募集情報を更新しました"
        );

        Ok(())
    }

    /// 募集のメッセージIDを更新
    pub async fn update_message_id(
        &self,
        txn: &DatabaseTransaction,
        recruitment_id: i32,
        message_id: DiscordMessageId,
    ) -> Result<()> {
        self.battle_recruitment_repo
            .update_message_id_with_txn(txn, recruitment_id, message_id)
            .await?;

        info!(
            recruitment_id = recruitment_id,
            message_id = message_id.get(),
            "募集メッセージIDを更新しました"
        );

        Ok(())
    }

    /// 規定人数到達通知フラグを更新
    pub async fn set_full_notification_sent(
        &self,
        txn: &DatabaseTransaction,
        recruitment_id: i32,
        sent: bool,
    ) -> Result<()> {
        self.battle_recruitment_repo
            .set_full_notification_sent_with_txn(txn, recruitment_id, sent)
            .await?;

        info!(
            recruitment_id = recruitment_id,
            sent = sent,
            "規定人数到達通知フラグを更新しました"
        );

        Ok(())
    }
}
