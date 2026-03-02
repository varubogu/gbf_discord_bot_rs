//! 自動募集コンポーネント操作Facade
//!
//! events層から呼び出される各種コンポーネント操作のDB更新/参照を担当する。

use crate::infrastructure::database::session::set_current_guild_id;
use crate::repository::QuestRepository;
use crate::repository::auto_recruitment::{
    UserDesiredQuestRepository,
    auto_recruitment_channel_repository::AutoRecruitmentChannelRepository,
};
use crate::types::{AppError, AppState, Result};
use sea_orm::TransactionTrait;
use std::collections::HashMap;
use tracing::{error, info};

/// 属性選択結果
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementSelectionResult {
    pub selected_battle_style_ids: Vec<i32>,
}

/// クエスト参加切り替え結果
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestJoinToggleResult {
    pub is_now_participating: bool,
}

/// 選択済みクエスト表示用データ
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedQuestItem {
    pub quest_name: String,
    pub battle_style_ids: Vec<i32>,
}

/// 日時チャンネル情報
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeChannelDate {
    pub month: i32,
    pub day: i32,
}

/// 属性選択を登録
pub async fn register_selected_elements(
    app_state: &AppState,
    guild_id: i64,
    user_id: i64,
    quest_id: i32,
    selected_battle_style_ids: Vec<i32>,
) -> Result<ElementSelectionResult> {
    let conn = app_state.guild_db();
    let txn = conn.begin().await?;
    set_current_guild_id(&txn, guild_id).await?;

    let result = async {
        let quest_repo = app_state.repositories.user_desired_quest;

        quest_repo
            .delete_all_styles(&txn, guild_id, user_id, quest_id)
            .await?;

        for battle_style_id in &selected_battle_style_ids {
            quest_repo
                .create(&txn, guild_id, user_id, quest_id, *battle_style_id)
                .await?;
        }

        info!(
            guild_id,
            user_id,
            quest_id,
            count = selected_battle_style_ids.len(),
            "属性を登録しました"
        );

        Ok(ElementSelectionResult {
            selected_battle_style_ids: selected_battle_style_ids.clone(),
        })
    }
    .await;

    match result {
        Ok(output) => {
            txn.commit().await?;
            Ok(output)
        }
        Err(e) => {
            txn.rollback().await?;
            error!(error = %e, guild_id, user_id, quest_id, "属性選択処理に失敗しました");
            Err(e)
        }
    }
}

/// 参加ボタンの状態を切り替える
pub async fn toggle_quest_join(
    app_state: &AppState,
    guild_id: i64,
    user_id: i64,
    quest_id: i32,
) -> Result<QuestJoinToggleResult> {
    let conn = app_state.guild_db();
    let txn = conn.begin().await?;
    set_current_guild_id(&txn, guild_id).await?;

    let result = async {
        let quest_repo = app_state.repositories.user_desired_quest;
        let existing = quest_repo
            .find_by_user(&txn, guild_id, user_id)
            .await?
            .into_iter()
            .filter(|q| q.quest_id == quest_id)
            .collect::<Vec<_>>();
        let is_participating = !existing.is_empty();

        if is_participating {
            quest_repo
                .delete_all_styles(&txn, guild_id, user_id, quest_id)
                .await?;
            info!(guild_id, user_id, quest_id, "クエスト参加を解除しました");
        } else {
            quest_repo
                .create(&txn, guild_id, user_id, quest_id, 0)
                .await?;
            info!(guild_id, user_id, quest_id, "クエスト参加を登録しました");
        }

        Ok(QuestJoinToggleResult {
            is_now_participating: !is_participating,
        })
    }
    .await;

    match result {
        Ok(output) => {
            txn.commit().await?;
            Ok(output)
        }
        Err(e) => {
            txn.rollback().await?;
            error!(error = %e, guild_id, user_id, quest_id, "クエスト参加処理に失敗しました");
            Err(e)
        }
    }
}

/// ユーザーの選択済みクエストを取得
pub async fn get_selected_quests(
    app_state: &AppState,
    guild_id: i64,
    user_id: i64,
) -> Result<Vec<SelectedQuestItem>> {
    let conn = app_state.guild_db();
    let txn = conn.begin().await?;
    set_current_guild_id(&txn, guild_id).await?;

    let result = async {
        let quest_repo = app_state.repositories.user_desired_quest;
        let master_quest_repo = app_state.repositories.quest;
        let user_quests = quest_repo.find_by_user(&txn, guild_id, user_id).await?;
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

        let quest_ids: Vec<i32> = quest_styles.keys().copied().collect();
        let all_quests = master_quest_repo.get_all(&txn).await?;
        let quest_map: HashMap<i32, String> = all_quests
            .into_iter()
            .filter(|q| quest_ids.contains(&q.id))
            .map(|q| (q.id, q.name))
            .collect();

        let mut selections = Vec::new();
        for (quest_id, styles) in quest_styles {
            let quest_name = quest_map
                .get(&quest_id)
                .cloned()
                .unwrap_or_else(|| "不明なクエスト".to_string());
            selections.push(SelectedQuestItem {
                quest_name,
                battle_style_ids: styles,
            });
        }

        Ok(selections)
    }
    .await;

    match result {
        Ok(output) => {
            txn.commit().await?;
            Ok(output)
        }
        Err(e) => {
            txn.rollback().await?;
            error!(error = %e, guild_id, user_id, "選択済みクエスト取得に失敗しました");
            Err(e)
        }
    }
}

/// 日時チャンネルIDから日付情報を取得
pub async fn get_time_channel_date(
    app_state: &AppState,
    guild_id: i64,
    channel_id: i64,
) -> Result<TimeChannelDate> {
    let conn = app_state.guild_db();
    let txn = conn.begin().await?;
    set_current_guild_id(&txn, guild_id).await?;

    let result = async {
        let channel_repo = app_state.repositories.auto_recruitment_channel;
        let channel = channel_repo
            .find_by_channel_id(&txn, guild_id, channel_id)
            .await?
            .ok_or_else(|| AppError::Generic("チャンネル情報が見つかりません".to_string()))?;

        Ok(TimeChannelDate {
            month: channel.month,
            day: channel.day,
        })
    }
    .await;

    match result {
        Ok(output) => {
            txn.commit().await?;
            Ok(output)
        }
        Err(e) => {
            txn.rollback().await?;
            error!(error = %e, guild_id, channel_id, "日時チャンネル情報取得に失敗しました");
            Err(e)
        }
    }
}
