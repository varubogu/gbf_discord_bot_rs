//! 募集参加者管理サービス（純粋なビジネスロジック）
//!
//! Discord API操作はfacade層で行う。
//! このファイルはDBアクセスとビジネスロジックのみを担当する。

use crate::types::discord::{DiscordChannelId, DiscordGuildId, DiscordMessageId};
use sea_orm::DatabaseTransaction;
use std::collections::HashMap;
use tracing::info;

use crate::models::battle_recruitments::BattleRecruitments;
use crate::repository::battle_recruitments_repository::BattleRecruitmentsRepository;
use crate::types::{AppError, Result};

/// ParticipantsService - 募集参加者管理を行うサービス
pub struct ParticipantsService<R: BattleRecruitmentsRepository> {
    battle_recruitment_repo: R,
}

impl<R: BattleRecruitmentsRepository> ParticipantsService<R> {
    /// 新しいParticipantsServiceを作成（依存性注入）
    pub fn new(battle_recruitment_repo: R) -> Self {
        Self {
            battle_recruitment_repo,
        }
    }

    /// 参加者をメッセージIDから更新する（Facade層用メソッド）
    pub async fn update_participants_by_message(
        &self,
        guild_id: u64,
        channel_id: u64,
        message_id: u64,
        txn: &DatabaseTransaction,
    ) -> Result<BattleRecruitments> {
        info!("ParticipantsService::update_participants_by_message - 参加者更新開始");

        // 募集情報の存在確認（トランザクション対応版を使用）
        // u64をドメイン型に変換
        let recruitment = self
            .battle_recruitment_repo
            .get_by_message_with_txn(
                txn,
                DiscordGuildId::new(guild_id),
                DiscordChannelId::new(channel_id),
                DiscordMessageId::new(message_id),
            )
            .await?
            .ok_or_else(|| AppError::NotFound("募集が見つかりませんでした".to_string()))?;

        // キャンセル済みの募集は処理を終了
        if recruitment.is_canceled {
            info!(
                recruitment_id = recruitment.id,
                "キャンセル済み募集のため処理をスキップします"
            );
            return Err(AppError::Business {
                message: "この募集はキャンセル済みです".to_string(),
            });
        }

        // 期限切れの募集は処理を終了
        let now = chrono::Utc::now();
        if recruitment.quest_start_at < now {
            info!(
                recruitment_id = recruitment.id,
                quest_start_at = %recruitment.quest_start_at,
                "期限切れ募集のため処理をスキップします"
            );
            return Err(AppError::Business {
                message: "この募集は期限切れです".to_string(),
            });
        }

        info!(recruitment_id = recruitment.id, "参加者更新処理完了");
        Ok(recruitment)
    }

    /// 一意の参加者数を取得（重複排除）
    /// 一人が複数のリアクションをしている場合は1人としてカウント
    pub fn count_unique_participants(
        &self,
        participants_by_reaction: &HashMap<String, Vec<String>>,
    ) -> usize {
        use std::collections::HashSet;

        let mut unique_participants = HashSet::new();
        for users in participants_by_reaction.values() {
            for user_mention in users {
                unique_participants.insert(user_mention.clone());
            }
        }

        unique_participants.len()
    }

    /// すべての参加者のメンションを取得（重複排除）
    pub fn get_all_participants(
        &self,
        participants_by_reaction: &HashMap<String, Vec<String>>,
    ) -> Vec<String> {
        use std::collections::HashSet;

        let mut unique_participants = HashSet::new();
        for users in participants_by_reaction.values() {
            for user_mention in users {
                unique_participants.insert(user_mention.clone());
            }
        }

        unique_participants.into_iter().collect()
    }
}
