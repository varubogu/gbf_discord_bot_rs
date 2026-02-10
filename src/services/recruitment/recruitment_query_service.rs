use crate::models::battle_recruitments::BattleRecruitments;
use crate::models::entities::master::battle_styles;
use crate::repository::BattleRecruitmentsRepository;
use crate::repository::BattleStyleRepository;
use crate::types::Result;
use crate::types::discord::{DiscordChannelId, DiscordGuildId, DiscordMessageId};
use sea_orm::DatabaseTransaction;
use tracing::debug;

/// 募集情報クエリService
/// 募集情報とBattleStyleの取得を担当
pub struct RecruitmentQueryService<BS, BR>
where
    BS: BattleStyleRepository,
    BR: BattleRecruitmentsRepository,
{
    battle_style_repo: BS,
    battle_recruitment_repo: BR,
}

impl<BS, BR> RecruitmentQueryService<BS, BR>
where
    BS: BattleStyleRepository,
    BR: BattleRecruitmentsRepository,
{
    pub fn new(battle_style_repo: BS, battle_recruitment_repo: BR) -> Self {
        Self {
            battle_style_repo,
            battle_recruitment_repo,
        }
    }

    /// メッセージIDから募集情報を取得
    pub async fn get_recruitment_by_message(
        &self,
        txn: &DatabaseTransaction,
        guild_id: u64,
        channel_id: u64,
        message_id: u64,
    ) -> Result<Option<BattleRecruitments>> {
        // u64をドメイン型に変換
        let recruitment = self
            .battle_recruitment_repo
            .get_by_message_with_txn(
                txn,
                DiscordGuildId::new(guild_id),
                DiscordChannelId::new(channel_id),
                DiscordMessageId::new(message_id),
            )
            .await?;

        debug!(
            guild_id = guild_id,
            channel_id = channel_id,
            message_id = message_id,
            found = recruitment.is_some(),
            "メッセージIDから募集情報を取得しました"
        );

        Ok(recruitment)
    }

    /// BattleStyleをIDで取得
    pub async fn get_battle_style_by_id(
        &self,
        txn: &DatabaseTransaction,
        battle_style_id: i32,
    ) -> Result<Option<battle_styles::Model>> {
        let battle_style = self
            .battle_style_repo
            .get_by_id(txn, battle_style_id)
            .await?;

        debug!(
            battle_style_id = battle_style_id,
            found = battle_style.is_some(),
            "BattleStyle情報を取得しました"
        );

        Ok(battle_style)
    }
}
