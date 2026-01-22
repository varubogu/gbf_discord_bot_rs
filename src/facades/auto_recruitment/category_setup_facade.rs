//! 自動募集カテゴリ設定Facade
//!
//! カテゴリ登録/解除/日数変更の処理を行う

use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::models::entities::worker::scheduled_tasks::ScheduledTaskType;
use crate::repository::auto_recruitment::{
    AutoRecruitmentChannelRepository, AutoRecruitmentRepository, CreateAutoRecruitmentParams,
};
use crate::repository::database::auto_recruitment::{
    SeaOrmAutoRecruitmentChannelRepository, SeaOrmAutoRecruitmentRepository,
};
use crate::repository::database::schedule::SeaOrmScheduledTaskRepository;
use crate::repository::schedule::ScheduledTaskRepository;
use crate::services::message::MessageTextId;
use crate::types::{AppError, AppState, Result};
use chrono::{Datelike, Duration, Utc};
use poise::serenity_prelude::{
    ChannelId, ChannelType, Context, CreateActionRow, CreateChannel, CreateMessage,
    CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption, GuildId, Http,
};
use rust_i18n::t;
use sea_orm::TransactionTrait;
use std::sync::Arc;
use tracing::{debug, error, info, instrument};

/// カテゴリ登録結果
pub struct CategoryRegistrationResult {
    /// カテゴリチャンネルID
    pub category_id: u64,
    /// 作成された日時チャンネル数
    pub channel_count: usize,
}

/// カテゴリを自動募集に登録
///
/// # 引数
/// * `ctx` - Discord Context
/// * `app_state` - アプリケーション状態
/// * `guild_id` - ギルドID
/// * `category_id` - カテゴリチャンネルID
/// * `days` - 募集日数（2-7日）
/// * `matching_channel_id` - マッチング通知チャンネルID（省略可能）
/// * `quest_channel_id` - クエスト選択チャンネルID（省略可能）
#[instrument(level = "info", skip(ctx, app_state))]
pub async fn register_category(
    ctx: &Context,
    app_state: &AppState,
    guild_id: u64,
    category_id: u64,
    days: i32,
    matching_channel_id: Option<u64>,
    quest_channel_id: Option<u64>,
) -> Result<CategoryRegistrationResult> {
    info!(guild_id, category_id, days, "自動募集カテゴリを登録します");

    // 日数の検証
    if !(2..=7).contains(&days) {
        return Err(AppError::Business {
            message: "募集日数は2〜7日の範囲で指定してください".to_string(),
        });
    }

    let conn = app_state.guild_db();
    let txn = conn.begin().await?;

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id as i64).await?;

    let result = async {
        let auto_recruitment_repo = SeaOrmAutoRecruitmentRepository::new();
        let channel_repo = SeaOrmAutoRecruitmentChannelRepository::new();

        // 既存の登録をチェック
        if auto_recruitment_repo
            .find_by_guild_id(&txn, guild_id as i64)
            .await?
            .is_some()
        {
            return Err(AppError::Business {
                message: "このギルドには既に自動募集が登録されています。先に解除してください。"
                    .to_string(),
            });
        }

        // auto_recruitmentsテーブルに登録
        let params = CreateAutoRecruitmentParams {
            guild_id: guild_id as i64,
            category_id: category_id as i64,
            matching_channel_id: matching_channel_id.map(|id| id as i64),
            quest_channel_id: quest_channel_id.map(|id| id as i64),
            days_range: days,
        };

        let auto_recruitment = auto_recruitment_repo.create(&txn, params).await?;

        info!(
            guild_id = auto_recruitment.guild_id,
            "自動募集設定を登録しました"
        );

        // 日時チャンネルを作成
        let created_channels =
            create_date_channels(&ctx.http, guild_id, category_id, days, &channel_repo, &txn)
                .await?;

        // ローテーションタスクを初期登録（翌日0時）
        let task_repo = SeaOrmScheduledTaskRepository::new();
        create_initial_rotation_task(&task_repo, &txn).await?;

        Ok(CategoryRegistrationResult {
            category_id,
            channel_count: created_channels,
        })
    }
    .await;

    match result {
        Ok(res) => {
            txn.commit().await?;
            info!(
                guild_id,
                category_id,
                channel_count = res.channel_count,
                "自動募集カテゴリを登録しました"
            );
            Ok(res)
        }
        Err(e) => {
            txn.rollback().await?;
            error!(error = %e, guild_id, category_id, "自動募集カテゴリの登録に失敗しました");
            Err(e)
        }
    }
}

/// カテゴリの自動募集を解除
///
/// # 引数
/// * `ctx` - Discord Context
/// * `app_state` - アプリケーション状態
/// * `guild_id` - ギルドID
#[instrument(level = "info", skip(ctx, app_state))]
pub async fn unregister_category(ctx: &Context, app_state: &AppState, guild_id: u64) -> Result<()> {
    info!(guild_id, "自動募集カテゴリを解除します");

    let conn = app_state.guild_db();
    let txn = conn.begin().await?;

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id as i64).await?;

    let result = async {
        let auto_recruitment_repo = SeaOrmAutoRecruitmentRepository::new();
        let channel_repo = SeaOrmAutoRecruitmentChannelRepository::new();

        // 自動募集設定を取得
        let _auto_recruitment = auto_recruitment_repo
            .find_by_guild_id(&txn, guild_id as i64)
            .await?
            .ok_or_else(|| AppError::Business {
                message: "このギルドには自動募集が登録されていません".to_string(),
            })?;

        // 日時チャンネルを削除（Discordからも削除）
        let channels = channel_repo.find_by_guild_id(&txn, guild_id as i64).await?;

        for channel in channels {
            let channel_id = ChannelId::new(channel.channel_id as u64);
            if let Err(e) = channel_id.delete(&ctx.http).await {
                error!(
                    channel_id = channel.channel_id,
                    error = %e,
                    "Discordチャンネルの削除に失敗しました"
                );
                // 失敗しても続行
            }
        }

        // DBから削除
        channel_repo
            .delete_all_by_guild_id(&txn, guild_id as i64)
            .await?;
        auto_recruitment_repo.delete(&txn, guild_id as i64).await?;

        Ok(())
    }
    .await;

    match result {
        Ok(()) => {
            txn.commit().await?;
            info!(guild_id, "自動募集カテゴリを解除しました");
            Ok(())
        }
        Err(e) => {
            txn.rollback().await?;
            error!(error = %e, guild_id, "自動募集カテゴリの解除に失敗しました");
            Err(e)
        }
    }
}

/// 募集日数を変更
///
/// # 引数
/// * `ctx` - Discord Context
/// * `app_state` - アプリケーション状態
/// * `guild_id` - ギルドID
/// * `new_days` - 新しい募集日数（2-7日）
#[instrument(level = "info", skip(ctx, app_state))]
pub async fn change_days(
    ctx: &Context,
    app_state: &AppState,
    guild_id: u64,
    new_days: i32,
) -> Result<()> {
    info!(guild_id, new_days, "自動募集の募集日数を変更します");

    // 日数の検証
    if !(2..=7).contains(&new_days) {
        return Err(AppError::Business {
            message: "募集日数は2〜7日の範囲で指定してください".to_string(),
        });
    }

    let conn = app_state.guild_db();
    let txn = conn.begin().await?;

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id as i64).await?;

    let result = async {
        let auto_recruitment_repo = SeaOrmAutoRecruitmentRepository::new();
        let channel_repo = SeaOrmAutoRecruitmentChannelRepository::new();

        // 自動募集設定を取得
        let auto_recruitment = auto_recruitment_repo
            .find_by_guild_id(&txn, guild_id as i64)
            .await?
            .ok_or_else(|| AppError::Business {
                message: "このギルドには自動募集が登録されていません".to_string(),
            })?;

        let current_days = auto_recruitment.days_range;

        if new_days == current_days {
            return Err(AppError::Business {
                message: format!("募集日数は既に{}日です", new_days),
            });
        }

        // 既存のチャンネルを取得
        let existing_channels = channel_repo.find_by_guild_id(&txn, guild_id as i64).await?;

        let category_id = auto_recruitment.category_id as u64;

        if new_days > current_days {
            // 増やす場合：追加のチャンネルを作成
            let channels_to_add = (new_days - current_days) as usize;
            let last_channel_date = if let Some(last) = existing_channels.last() {
                let now_jst = Utc::now() + Duration::hours(9);
                let today = now_jst.date_naive();
                chrono::NaiveDate::from_ymd_opt(today.year(), last.month as u32, last.day as u32)
                    .unwrap_or(today)
            } else {
                let now_jst = Utc::now() + Duration::hours(9);
                now_jst.date_naive()
            };

            let discord_guild_id = GuildId::new(guild_id);

            for i in 1..=channels_to_add {
                let new_date = last_channel_date + Duration::days(i as i64);
                let channel_name = format!("{}月{}日", new_date.month(), new_date.day());
                let sort_order = (existing_channels.len() + i) as i32;

                // Discordチャンネルを作成（カテゴリの権限を継承）
                let channel = discord_guild_id
                    .create_channel(
                        &ctx.http,
                        CreateChannel::new(&channel_name)
                            .kind(ChannelType::Text)
                            .category(ChannelId::new(category_id)),
                    )
                    .await
                    .map_err(|e| {
                        error!(error = %e, guild_id, category_id, "チャンネルの作成に失敗しました");
                        AppError::ChannelCreationFailed
                    })?;

                // 時間選択コンポーネントを送信
                send_time_selection_message(&ctx.http, channel.id).await?;

                // DBに登録
                channel_repo
                    .create(
                        &txn,
                        guild_id as i64,
                        channel.id.get() as i64,
                        new_date.month() as i32,
                        new_date.day() as i32,
                        sort_order,
                    )
                    .await?;
            }
        } else {
            // 減らす場合：末尾のチャンネルを削除
            let channels_to_remove = (current_days - new_days) as usize;
            let channels_to_delete: Vec<_> = existing_channels
                .iter()
                .rev()
                .take(channels_to_remove)
                .collect();

            for channel in channels_to_delete {
                // Discordチャンネルを削除
                let channel_id = ChannelId::new(channel.channel_id as u64);
                if let Err(e) = channel_id.delete(&ctx.http).await {
                    error!(
                        channel_id = channel.channel_id,
                        error = %e,
                        "Discordチャンネルの削除に失敗しました"
                    );
                    // 失敗しても続行
                }

                // DBから削除
                channel_repo
                    .delete_by_channel_id(&txn, guild_id as i64, channel.channel_id)
                    .await?;
            }
        }

        // 日数を更新
        auto_recruitment_repo
            .update_days_range(&txn, guild_id as i64, new_days)
            .await?;

        Ok(())
    }
    .await;

    match result {
        Ok(()) => {
            txn.commit().await?;
            info!(guild_id, new_days, "自動募集の募集日数を変更しました");
            Ok(())
        }
        Err(e) => {
            txn.rollback().await?;
            error!(error = %e, guild_id, "自動募集の募集日数の変更に失敗しました");
            Err(e)
        }
    }
}

/// 日時チャンネルを作成
async fn create_date_channels<C: AutoRecruitmentChannelRepository>(
    http: &Arc<Http>,
    guild_id: u64,
    category_id: u64,
    days: i32,
    channel_repo: &C,
    txn: &sea_orm::DatabaseTransaction,
) -> Result<usize> {
    debug!(guild_id, category_id, days, "日時チャンネルを作成します");

    // 今日の日付を取得（JST）
    let now_utc = Utc::now();
    let now_jst = now_utc + Duration::hours(9);
    let today = now_jst.date_naive();

    let discord_guild_id = GuildId::new(guild_id);
    let mut created_count = 0;

    for i in 0..days {
        let date = today + Duration::days(i as i64);
        let channel_name = format!("{}月{}日", date.month(), date.day());

        // Discordチャンネルを作成（カテゴリの権限を継承）
        let channel = discord_guild_id
            .create_channel(
                http,
                CreateChannel::new(&channel_name)
                    .kind(ChannelType::Text)
                    .category(ChannelId::new(category_id)),
            )
            .await
            .map_err(|e| {
                error!(error = %e, guild_id, category_id, "チャンネルの作成に失敗しました");
                AppError::ChannelCreationFailed
            })?;

        // 時間選択コンポーネントを送信
        send_time_selection_message(http, channel.id).await?;

        // DBに登録
        channel_repo
            .create(
                txn,
                guild_id as i64,
                channel.id.get() as i64,
                date.month() as i32,
                date.day() as i32,
                i,
            )
            .await?;

        created_count += 1;
        debug!(
            channel_id = channel.id.get(),
            channel_name, "日時チャンネルを作成しました"
        );
    }

    Ok(created_count)
}

/// 時間選択メッセージを送信
///
/// グラブルではAM5:00に日付が変わるため、1/21チャンネルは「1/21 5:00〜1/22 4:00」を対象とする。
/// 選択肢は降順（夜の時間帯が先）で表示し、翌日分は「翌0:00」のように表記する。
/// 内部値は0-28（5-23は当日、24-28は翌日0-4時を表す）。
async fn send_time_selection_message(http: &Arc<Http>, channel_id: ChannelId) -> Result<()> {
    // ゲーム内日付: 当日5:00〜翌日4:00（24時間）
    // 降順で表示: 翌4:00, 翌3:00, 翌2:00, 翌1:00, 翌0:00, 23:00, 22:00, ..., 5:00
    let mut options: Vec<CreateSelectMenuOption> = Vec::with_capacity(24);

    // 翌日分（4:00→0:00の降順）- 内部値は28, 27, 26, 25, 24
    for hour in (0..=4).rev() {
        let label = format!("翌{hour}:00");
        let value = (24 + hour).to_string(); // 24-28
        options.push(CreateSelectMenuOption::new(label, value));
    }

    // 当日分（23:00→5:00の降順）- 内部値は23, 22, ..., 5
    for hour in (5..=23).rev() {
        let label = format!("{hour}:00");
        let value = hour.to_string();
        options.push(CreateSelectMenuOption::new(label, value));
    }

    // 多言語対応のplaceholderを取得（デフォルトは日本語）
    let placeholder = t!(
        MessageTextId::AutoRecruitmentTimeSelectPlaceholder.as_str(),
        locale = "ja"
    )
    .to_string();

    let select_menu = CreateSelectMenu::new(
        "auto_recruit_time",
        CreateSelectMenuKind::String { options },
    )
    .placeholder(&placeholder)
    .min_values(0)
    .max_values(24);

    let message = CreateMessage::new()
        .content("**参加可能な時間帯を選択してください**\n複数選択可能です。選択を変更すると自動的に更新されます。")
        .components(vec![CreateActionRow::SelectMenu(select_menu)]);

    channel_id.send_message(http, message).await.map_err(|e| {
        error!(error = %e, channel_id = channel_id.get(), "時間選択メッセージの送信に失敗しました");
        AppError::Business {
            message: "時間選択メッセージの送信に失敗しました".to_string(),
        }
    })?;

    Ok(())
}

/// 初期ローテーションタスクを作成
async fn create_initial_rotation_task(
    task_repo: &SeaOrmScheduledTaskRepository,
    txn: &sea_orm::DatabaseTransaction,
) -> Result<()> {
    // 翌日の0時（JST）をUTCに変換
    let now_utc = Utc::now();
    let now_jst = now_utc + Duration::hours(9);
    let tomorrow_jst = (now_jst + Duration::days(1))
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap();
    // JST 0時 = UTC 15時（前日）
    let next_execution_utc = tomorrow_jst - Duration::hours(9);
    let next_execution =
        chrono::DateTime::<Utc>::from_naive_utc_and_offset(next_execution_utc, Utc);

    // 既存のローテーションタスクがあるか確認（重複防止）
    let pending_tasks = task_repo
        .find_pending_to(txn, next_execution + Duration::days(1))
        .await?;

    let has_rotation_task = pending_tasks
        .iter()
        .any(|t| t.task_type == ScheduledTaskType::AutoRecruitmentRotation as i32);

    if !has_rotation_task {
        task_repo
            .create(
                txn,
                next_execution,
                ScheduledTaskType::AutoRecruitmentRotation as i32,
                None,
                None,
            )
            .await?;
        info!(
            next_execution = %next_execution,
            "初期ローテーションタスクを作成しました"
        );
    }

    Ok(())
}
