use crate::repository::QuestRepository;
use crate::repository::auto_recruitment::{
    AutoRecruitmentChannelRepository, UserDesiredQuestRepository,
};
use crate::types::{AppError, Result};
use sea_orm::DatabaseTransaction;
use std::collections::HashMap;
use tracing::info;

/// 選択済みクエスト情報
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedQuestData {
    pub quest_name: String,
    pub battle_style_ids: Vec<i32>,
}

/// 日時チャンネル日付情報
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeChannelDateData {
    pub month: i32,
    pub day: i32,
}

/// 自動募集コンポーネント操作サービス
///
/// ユーザー選択状態の更新・取得を担当する。
pub struct InteractionService<UQ, Q, AC>
where
    UQ: UserDesiredQuestRepository,
    Q: QuestRepository,
    AC: AutoRecruitmentChannelRepository,
{
    user_desired_quest_repo: UQ,
    quest_repo: Q,
    channel_repo: AC,
}

impl<UQ, Q, AC> InteractionService<UQ, Q, AC>
where
    UQ: UserDesiredQuestRepository,
    Q: QuestRepository,
    AC: AutoRecruitmentChannelRepository,
{
    pub fn new(user_desired_quest_repo: UQ, quest_repo: Q, channel_repo: AC) -> Self {
        Self {
            user_desired_quest_repo,
            quest_repo,
            channel_repo,
        }
    }

    /// クエストに対する属性選択を全置換
    pub async fn replace_selected_elements(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        user_id: i64,
        quest_id: i32,
        selected_battle_style_ids: &[i32],
    ) -> Result<()> {
        self.user_desired_quest_repo
            .delete_all_styles(txn, guild_id, user_id, quest_id)
            .await?;

        for battle_style_id in selected_battle_style_ids {
            self.user_desired_quest_repo
                .create(txn, guild_id, user_id, quest_id, *battle_style_id)
                .await?;
        }

        info!(
            guild_id = guild_id,
            user_id = user_id,
            quest_id = quest_id,
            count = selected_battle_style_ids.len(),
            "属性選択を更新しました"
        );

        Ok(())
    }

    /// クエスト参加状態を切り替える
    ///
    /// 戻り値は切り替え後に参加中かどうか。
    pub async fn toggle_quest_join(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        user_id: i64,
        quest_id: i32,
    ) -> Result<bool> {
        let existing = self
            .user_desired_quest_repo
            .find_by_user(txn, guild_id, user_id)
            .await?
            .into_iter()
            .filter(|q| q.quest_id == quest_id)
            .collect::<Vec<_>>();
        let is_participating = !existing.is_empty();

        if is_participating {
            self.user_desired_quest_repo
                .delete_all_styles(txn, guild_id, user_id, quest_id)
                .await?;
            info!(
                guild_id = guild_id,
                user_id = user_id,
                quest_id = quest_id,
                "クエスト参加を解除しました"
            );
        } else {
            self.user_desired_quest_repo
                .create(txn, guild_id, user_id, quest_id, 0)
                .await?;
            info!(
                guild_id = guild_id,
                user_id = user_id,
                quest_id = quest_id,
                "クエスト参加を登録しました"
            );
        }

        Ok(!is_participating)
    }

    /// ユーザーの選択済みクエスト一覧を取得
    pub async fn get_selected_quests(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        user_id: i64,
    ) -> Result<Vec<SelectedQuestData>> {
        let user_quests = self
            .user_desired_quest_repo
            .find_by_user(txn, guild_id, user_id)
            .await?;
        if user_quests.is_empty() {
            return Ok(vec![]);
        }

        let mut quest_styles: HashMap<i32, Vec<i32>> = HashMap::new();
        for user_quest in &user_quests {
            quest_styles
                .entry(user_quest.quest_id)
                .or_default()
                .push(user_quest.battle_style_id);
        }

        let all_quests = self.quest_repo.get_all(txn).await?;
        let quest_map: HashMap<i32, String> =
            all_quests.into_iter().map(|q| (q.id, q.name)).collect();

        let mut selections = Vec::new();
        for (quest_id, styles) in quest_styles {
            let quest_name = quest_map
                .get(&quest_id)
                .cloned()
                .unwrap_or_else(|| "不明なクエスト".to_string());
            selections.push(SelectedQuestData {
                quest_name,
                battle_style_ids: styles,
            });
        }

        Ok(selections)
    }

    /// 日時チャンネルIDから日付情報を取得
    pub async fn get_time_channel_date(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        channel_id: i64,
    ) -> Result<TimeChannelDateData> {
        let channel = self
            .channel_repo
            .find_by_channel_id(txn, guild_id, channel_id)
            .await?
            .ok_or_else(|| AppError::Generic("チャンネル情報が見つかりません".to_string()))?;

        Ok(TimeChannelDateData {
            month: channel.month,
            day: channel.day,
        })
    }
}
