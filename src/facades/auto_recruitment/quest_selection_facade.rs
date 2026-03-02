//! クエスト選択Facade
//!
//! ユーザーのクエスト選択を処理する

use crate::infrastructure::database::session::set_current_guild_id;
use crate::services::auto_recruitment::quest_selection_service::QuestSelectionService;
use crate::types::{AppState, Result};
use sea_orm::TransactionTrait;
use tracing::{error, info, instrument};

/// クエスト選択結果
#[derive(Debug)]
pub enum QuestSelectionResult {
    /// 登録完了
    Registered { quest_ids: Vec<i32> },
}

/// クエスト選択を処理
///
/// # 引数
/// * `app_state` - アプリケーション状態
/// * `guild_id` - ギルドID
/// * `user_id` - ユーザーID
/// * `quest_ids` - 選択されたクエストIDのリスト
#[instrument(level = "info", skip(app_state))]
pub async fn handle_quest_selection(
    app_state: &AppState,
    guild_id: u64,
    user_id: u64,
    quest_ids: Vec<i32>,
) -> Result<QuestSelectionResult> {
    info!(
        guild_id,
        user_id,
        quest_count = quest_ids.len(),
        "クエスト選択を処理します"
    );

    let conn = app_state.guild_db();
    let txn = conn.begin().await?;

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id as i64).await?;

    let result = async {
        let quest_selection_service = QuestSelectionService::new(
            app_state.repositories.auto_recruitment,
            app_state.repositories.user_desired_quest,
        );

        // 自動募集設定を確認
        quest_selection_service
            .ensure_auto_recruitment_exists(&txn, guild_id as i64)
            .await?;

        // 既存のクエスト選択を削除して新しく登録
        quest_selection_service
            .replace_user_desired_quests(&txn, guild_id as i64, user_id as i64, &quest_ids)
            .await?;

        info!(
            guild_id,
            user_id,
            quest_ids = ?quest_ids,
            "クエスト選択を登録しました"
        );

        // TODO: マッチングチェックは時間選択時に実行

        Ok(QuestSelectionResult::Registered {
            quest_ids: quest_ids.clone(),
        })
    }
    .await;

    match result {
        Ok(res) => {
            txn.commit().await?;
            info!(guild_id, user_id, "クエスト選択処理が完了しました");
            Ok(res)
        }
        Err(e) => {
            txn.rollback().await?;
            error!(error = %e, guild_id, user_id, "クエスト選択処理に失敗しました");
            Err(e)
        }
    }
}
