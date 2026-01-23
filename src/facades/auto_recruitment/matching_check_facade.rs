//! マッチングチェックFacade
//!
//! 時間選択・クエスト選択後のマッチングチェックと通知を行う

use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::repository::auto_recruitment::{
    AutoRecruitmentRepository, MatchedRecruitmentChannelRepository,
};
use crate::repository::database::auto_recruitment::{
    SeaOrmAutoRecruitmentParticipantRepository, SeaOrmAutoRecruitmentRepository,
    SeaOrmMatchedRecruitmentChannelRepository, SeaOrmUserDesiredQuestRepository,
};
use crate::repository::database::quest_repository::SeaOrmQuestRepository;
use crate::repository::quest_repository::QuestRepository;
use crate::services::auto_recruitment::matching_service::AutoMatchingService;
use crate::services::auto_recruitment::notification_service::AutoRecruitmentNotificationService;
use crate::types::{AppState, Result};
use poise::serenity_prelude::Context;
use sea_orm::TransactionTrait;
use std::sync::Arc;
use tracing::{debug, error, info};

/// 時間選択後のマッチングチェックと通知
///
/// # 処理フロー
/// 1. 登録した各時間についてマッチングをチェック
/// 2. マッチングがあれば通知を送信
pub async fn check_and_notify_after_time_selection(
    ctx: &Context,
    app_state: &AppState,
    guild_id: u64,
    user_id: u64,
    month: i32,
    day: i32,
    hours: Vec<i32>,
) -> Result<()> {
    info!(
        guild_id,
        user_id,
        month,
        day,
        hour_count = hours.len(),
        "時間選択後のマッチングチェックを開始"
    );

    let conn = app_state.guild_db();
    let txn = conn.begin().await?;
    set_current_guild_id(&txn, guild_id as i64).await?;

    let participant_repo = Arc::new(SeaOrmAutoRecruitmentParticipantRepository::new());
    let user_quest_repo = Arc::new(SeaOrmUserDesiredQuestRepository::new());
    let matched_repo = Arc::new(SeaOrmMatchedRecruitmentChannelRepository::new());
    let auto_recruitment_repo = SeaOrmAutoRecruitmentRepository::new();
    let quest_repo = SeaOrmQuestRepository::new();

    let matching_service = AutoMatchingService::new(
        participant_repo,
        user_quest_repo.clone(),
        matched_repo.clone(),
    );
    let notification_service = AutoRecruitmentNotificationService::new();

    // 自動募集設定を取得（マッチングチャンネルIDが必要）
    let auto_recruitment = match auto_recruitment_repo
        .find_by_guild_id(&txn, guild_id as i64)
        .await?
    {
        Some(ar) => ar,
        None => {
            debug!(guild_id, "自動募集設定が見つかりません");
            txn.commit().await?;
            return Ok(());
        }
    };

    let matching_channel_id = match auto_recruitment.matching_channel_id {
        Some(id) => id as u64,
        None => {
            debug!(guild_id, "マッチングチャンネルが設定されていません");
            txn.commit().await?;
            return Ok(());
        }
    };

    // 各時間についてマッチングをチェック
    for hour in hours {
        match matching_service
            .check_match_by_time(&txn, guild_id as i64, user_id as i64, month, day, hour)
            .await
        {
            Ok(Some(match_result)) => {
                info!(
                    guild_id,
                    month,
                    day,
                    hour,
                    user_count = match_result.user_ids.len(),
                    "マッチング検出"
                );

                // クエスト名を取得
                let mut quest_candidates = Vec::new();
                for quest_id in &match_result.common_quest_ids {
                    if let Ok(Some(quest)) = quest_repo.get_by_target_id(&txn, *quest_id).await {
                        quest_candidates.push((*quest_id, quest.name.clone()));
                    }
                }

                if quest_candidates.is_empty() {
                    debug!(guild_id, "共通クエストの詳細が取得できませんでした");
                    continue;
                }

                // 既存のマッチングがあれば使用、なければ新規作成
                let matched_id = if let Some(existing_id) = match_result.existing_matched_id {
                    existing_id
                } else {
                    // 先にレコードを作成（message_id=0）
                    let matched = matched_repo
                        .create(
                            &txn,
                            guild_id as i64,
                            matching_channel_id as i64,
                            0, // message_idは後で更新
                            month,
                            day,
                            hour,
                        )
                        .await?;
                    matched.id
                };

                // 通知を送信
                let participants: Vec<u64> =
                    match_result.user_ids.iter().map(|id| *id as u64).collect();

                match notification_service
                    .notify_match(
                        &ctx.http,
                        matching_channel_id,
                        &participants,
                        &quest_candidates,
                        month,
                        day,
                        hour,
                        matched_id,
                    )
                    .await
                {
                    Ok(message) => {
                        // メッセージIDを更新（新規作成の場合のみ）
                        if match_result.existing_matched_id.is_none() {
                            if let Err(e) = matched_repo
                                .update_message_id(&txn, matched_id, message.id.get() as i64)
                                .await
                            {
                                error!(error = %e, guild_id, "メッセージIDの更新に失敗しました");
                            }
                        }
                    }
                    Err(e) => {
                        error!(error = %e, guild_id, "マッチング通知の送信に失敗しました");
                    }
                }
            }
            Ok(None) => {
                debug!(guild_id, month, day, hour, "マッチングなし");
            }
            Err(e) => {
                error!(error = %e, guild_id, month, day, hour, "マッチングチェックに失敗しました");
            }
        }
    }

    txn.commit().await?;
    info!(guild_id, user_id, "時間選択後のマッチングチェックが完了");
    Ok(())
}

/// クエスト選択後のマッチングチェックと通知
///
/// # 処理フロー
/// 1. 登録した各クエストについてマッチングをチェック
/// 2. マッチングがあれば通知を送信
pub async fn check_and_notify_after_quest_selection(
    ctx: &Context,
    app_state: &AppState,
    guild_id: u64,
    user_id: u64,
    quest_ids: Vec<i32>,
) -> Result<()> {
    info!(
        guild_id,
        user_id,
        quest_count = quest_ids.len(),
        "クエスト選択後のマッチングチェックを開始"
    );

    let conn = app_state.guild_db();
    let txn = conn.begin().await?;
    set_current_guild_id(&txn, guild_id as i64).await?;

    let participant_repo = Arc::new(SeaOrmAutoRecruitmentParticipantRepository::new());
    let user_quest_repo = Arc::new(SeaOrmUserDesiredQuestRepository::new());
    let matched_repo = Arc::new(SeaOrmMatchedRecruitmentChannelRepository::new());
    let auto_recruitment_repo = SeaOrmAutoRecruitmentRepository::new();
    let quest_repo = SeaOrmQuestRepository::new();

    let matching_service = AutoMatchingService::new(
        participant_repo,
        user_quest_repo.clone(),
        matched_repo.clone(),
    );
    let notification_service = AutoRecruitmentNotificationService::new();

    // 自動募集設定を取得（マッチングチャンネルIDが必要）
    let auto_recruitment = match auto_recruitment_repo
        .find_by_guild_id(&txn, guild_id as i64)
        .await?
    {
        Some(ar) => ar,
        None => {
            debug!(guild_id, "自動募集設定が見つかりません");
            txn.commit().await?;
            return Ok(());
        }
    };

    let matching_channel_id = match auto_recruitment.matching_channel_id {
        Some(id) => id as u64,
        None => {
            debug!(guild_id, "マッチングチャンネルが設定されていません");
            txn.commit().await?;
            return Ok(());
        }
    };

    // 重複チェック用のSet
    let mut notified_datetimes = std::collections::HashSet::new();

    // 各クエストについてマッチングをチェック
    for quest_id in quest_ids {
        match matching_service
            .check_match_by_quest(&txn, guild_id as i64, user_id as i64, quest_id)
            .await
        {
            Ok(match_results) => {
                for match_result in match_results {
                    // 同じ日時で既に通知済みならスキップ
                    let datetime_key = (match_result.month, match_result.day, match_result.hour);
                    if notified_datetimes.contains(&datetime_key) {
                        continue;
                    }
                    notified_datetimes.insert(datetime_key);

                    info!(
                        guild_id,
                        month = match_result.month,
                        day = match_result.day,
                        hour = match_result.hour,
                        user_count = match_result.user_ids.len(),
                        "マッチング検出"
                    );

                    // クエスト名を取得
                    let mut quest_candidates = Vec::new();
                    for qid in &match_result.common_quest_ids {
                        if let Ok(Some(quest)) = quest_repo.get_by_target_id(&txn, *qid).await {
                            quest_candidates.push((*qid, quest.name.clone()));
                        }
                    }

                    if quest_candidates.is_empty() {
                        debug!(guild_id, "共通クエストの詳細が取得できませんでした");
                        continue;
                    }

                    // 既存のマッチングがあれば使用、なければ新規作成
                    let matched_id = if let Some(existing_id) = match_result.existing_matched_id {
                        existing_id
                    } else {
                        // 先にレコードを作成（message_id=0）
                        let matched = matched_repo
                            .create(
                                &txn,
                                guild_id as i64,
                                matching_channel_id as i64,
                                0, // message_idは後で更新
                                match_result.month,
                                match_result.day,
                                match_result.hour,
                            )
                            .await?;
                        matched.id
                    };

                    // 通知を送信
                    let participants: Vec<u64> =
                        match_result.user_ids.iter().map(|id| *id as u64).collect();

                    match notification_service
                        .notify_match(
                            &ctx.http,
                            matching_channel_id,
                            &participants,
                            &quest_candidates,
                            match_result.month,
                            match_result.day,
                            match_result.hour,
                            matched_id,
                        )
                        .await
                    {
                        Ok(message) => {
                            // メッセージIDを更新（新規作成の場合のみ）
                            if match_result.existing_matched_id.is_none() {
                                if let Err(e) = matched_repo
                                    .update_message_id(&txn, matched_id, message.id.get() as i64)
                                    .await
                                {
                                    error!(error = %e, guild_id, "メッセージIDの更新に失敗しました");
                                }
                            }
                        }
                        Err(e) => {
                            error!(error = %e, guild_id, "マッチング通知の送信に失敗しました");
                        }
                    }
                }
            }
            Err(e) => {
                error!(error = %e, guild_id, quest_id, "マッチングチェックに失敗しました");
            }
        }
    }

    txn.commit().await?;
    info!(
        guild_id,
        user_id, "クエスト選択後のマッチングチェックが完了"
    );
    Ok(())
}
