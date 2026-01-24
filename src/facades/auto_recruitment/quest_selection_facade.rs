//! クエスト選択Facade
//!
//! ユーザーのクエスト選択を処理する

use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::repository::auto_recruitment::{AutoRecruitmentRepository, UserDesiredQuestRepository};
use crate::repository::database::auto_recruitment::{
    SeaOrmAutoRecruitmentRepository, SeaOrmUserDesiredQuestRepository,
};
use crate::types::{AppError, AppState, Result};
use poise::serenity_prelude::Context;
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
/// * `_ctx` - Discord Context（将来の拡張用）
/// * `app_state` - アプリケーション状態
/// * `guild_id` - ギルドID
/// * `user_id` - ユーザーID
/// * `quest_ids` - 選択されたクエストIDのリスト
#[instrument(level = "info", skip(_ctx, app_state))]
pub async fn handle_quest_selection(
    _ctx: &Context,
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
        let auto_recruitment_repo = SeaOrmAutoRecruitmentRepository::new();
        let quest_repo = SeaOrmUserDesiredQuestRepository::new();

        // 自動募集設定を確認
        let _auto_recruitment = auto_recruitment_repo
            .find_by_guild_id(&txn, guild_id as i64)
            .await?
            .ok_or_else(|| AppError::Business {
                message: "このギルドには自動募集が登録されていません".to_string(),
            })?;

        // 既存のクエスト選択を削除して新しく登録
        quest_repo
            .delete_all_by_user(&txn, guild_id as i64, user_id as i64)
            .await?;

        // 属性指定なしクエストとしてbattle_style_id=0で登録
        // 6属性クエストは別途UIから属性を指定して登録する
        for quest_id in &quest_ids {
            quest_repo
                .create(&txn, guild_id as i64, user_id as i64, *quest_id, 0)
                .await?;
        }

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
