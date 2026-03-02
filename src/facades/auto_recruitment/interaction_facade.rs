//! 自動募集コンポーネント操作Facade
//!
//! events層から呼び出される各種コンポーネント操作のDB更新/参照を担当する。

use crate::infrastructure::database::session::set_current_guild_id;
use crate::services::auto_recruitment::InteractionService;
use crate::types::{AppState, Result};
use sea_orm::TransactionTrait;
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
        let service = InteractionService::new(
            app_state.repositories.user_desired_quest,
            app_state.repositories.quest,
            app_state.repositories.auto_recruitment_channel,
        );
        service
            .replace_selected_elements(
                &txn,
                guild_id,
                user_id,
                quest_id,
                &selected_battle_style_ids,
            )
            .await?;

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
        let service = InteractionService::new(
            app_state.repositories.user_desired_quest,
            app_state.repositories.quest,
            app_state.repositories.auto_recruitment_channel,
        );
        let is_now_participating = service
            .toggle_quest_join(&txn, guild_id, user_id, quest_id)
            .await?;

        Ok(QuestJoinToggleResult {
            is_now_participating,
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
        let service = InteractionService::new(
            app_state.repositories.user_desired_quest,
            app_state.repositories.quest,
            app_state.repositories.auto_recruitment_channel,
        );
        let selected_quests = service.get_selected_quests(&txn, guild_id, user_id).await?;
        Ok(selected_quests
            .into_iter()
            .map(|item| SelectedQuestItem {
                quest_name: item.quest_name,
                battle_style_ids: item.battle_style_ids,
            })
            .collect())
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
        let service = InteractionService::new(
            app_state.repositories.user_desired_quest,
            app_state.repositories.quest,
            app_state.repositories.auto_recruitment_channel,
        );
        let date = service
            .get_time_channel_date(&txn, guild_id, channel_id)
            .await?;
        Ok(TimeChannelDate {
            month: date.month,
            day: date.day,
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
