//! 投票処理Facade
//!
//! マッチング後のクエスト投票を処理する

use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::repository::auto_recruitment::MatchedRecruitmentChannelRepository;
use crate::repository::database::auto_recruitment::{
    SeaOrmMatchedRecruitmentChannelRepository, SeaOrmMatchedRecruitmentVoteRepository,
};
use crate::services::auto_recruitment::VotingService;
use crate::types::{AppError, AppState, Result};
use poise::serenity_prelude::Context;
use sea_orm::TransactionTrait;
use std::sync::Arc;
use tracing::{error, info, instrument};

/// 投票結果
#[derive(Debug)]
pub enum VotingResult {
    /// 投票受付完了
    Accepted,
}

/// 投票を処理
///
/// # 引数
/// * `_ctx` - Discord Context（将来の拡張用）
/// * `app_state` - アプリケーション状態
/// * `guild_id` - ギルドID
/// * `user_id` - ユーザーID
/// * `matched_id` - マッチングID
/// * `quest_id` - 選択したクエストID（Noneは「何でも良い」）
#[instrument(level = "info", skip(_ctx, app_state))]
pub async fn handle_vote(
    _ctx: &Context,
    app_state: &AppState,
    guild_id: u64,
    user_id: u64,
    matched_id: i32,
    quest_id: Option<i32>,
) -> Result<VotingResult> {
    info!(guild_id, user_id, matched_id, ?quest_id, "投票を処理します");

    let conn = app_state.guild_db();
    let txn = conn.begin().await?;

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id as i64).await?;

    let result = async {
        let vote_repo = Arc::new(SeaOrmMatchedRecruitmentVoteRepository::new());
        let matched_repo = Arc::new(SeaOrmMatchedRecruitmentChannelRepository::new());

        // マッチング情報を取得
        let matched = matched_repo
            .find_by_id(&txn, matched_id)
            .await?
            .ok_or_else(|| AppError::Business {
                message: "マッチング情報が見つかりません".to_string(),
            })?;

        // 既に決定済みかチェック
        if matched.is_decided {
            return Err(AppError::Business {
                message: "このマッチングは既にクエストが決定しています".to_string(),
            });
        }

        // 投票サービスを作成
        let voting_service = VotingService::new(vote_repo, matched_repo);

        // 投票を登録
        voting_service
            .vote(&txn, matched_id, user_id as i64, quest_id)
            .await?;

        info!(matched_id, user_id, ?quest_id, "投票を登録しました");

        // TODO: 投票結果集計とクエスト決定は別途実装

        Ok(VotingResult::Accepted)
    }
    .await;

    match result {
        Ok(res) => {
            txn.commit().await?;
            info!(guild_id, user_id, matched_id, "投票処理が完了しました");
            Ok(res)
        }
        Err(e) => {
            txn.rollback().await?;
            error!(error = %e, guild_id, user_id, matched_id, "投票処理に失敗しました");
            Err(e)
        }
    }
}
