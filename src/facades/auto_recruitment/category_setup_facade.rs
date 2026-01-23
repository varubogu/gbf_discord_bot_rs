//! 自動募集カテゴリ設定Facade
//!
//! カテゴリ登録/解除/日数変更の処理を行う

use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::models::entities::worker::scheduled_tasks::ScheduledTaskType;
use crate::repository::QuestRepository;
use crate::repository::auto_recruitment::{
    AutoRecruitmentChannelRepository, AutoRecruitmentRepository, CreateAutoRecruitmentParams,
};
use crate::repository::database::auto_recruitment::{
    SeaOrmAutoRecruitmentChannelRepository, SeaOrmAutoRecruitmentRepository,
};
use crate::repository::database::quest_repository::SeaOrmQuestRepository;
use crate::repository::database::schedule::SeaOrmScheduledTaskRepository;
use crate::repository::schedule::ScheduledTaskRepository;
use crate::services::auto_recruitment::ui::quest_select_menu::QuestSelectMenuBuilder;
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

        let discord_guild_id = GuildId::new(guild_id);

        // マッチングチャンネルの処理
        let (final_matching_channel_id, matching_is_bot_created, matching_message_id) =
            if let Some(ch_id) = matching_channel_id {
                // 指定されたチャンネルにメッセージを送信
                let channel_id = ChannelId::new(ch_id);
                let msg_id = send_matching_channel_message(&ctx.http, channel_id).await?;
                (ch_id, false, Some(msg_id))
            } else {
                // チャンネルを新規作成
                let channel = discord_guild_id
                    .create_channel(
                        &ctx.http,
                        CreateChannel::new("マッチング")
                            .kind(ChannelType::Text)
                            .category(ChannelId::new(category_id)),
                    )
                    .await
                    .map_err(|e| {
                        error!(error = %e, guild_id, "マッチングチャンネルの作成に失敗しました");
                        AppError::ChannelCreationFailed
                    })?;
                let msg_id = send_matching_channel_message(&ctx.http, channel.id).await?;
                (channel.id.get(), true, Some(msg_id))
            };

        // クエスト一覧を取得（有効なクエストのみ）
        let quest_repo = SeaOrmQuestRepository::new();
        let quests = quest_repo
            .search_enabled_quests(&txn, guild_id as i64, "")
            .await?;

        // クエストリストを(id, name)のタプルに変換
        let quest_list: Vec<(i32, String)> = quests
            .iter()
            .map(|q| (q.quest_id, q.name.clone()))
            .collect();

        // クエストチャンネルの処理
        let (final_quest_channel_id, quest_is_bot_created, quest_message_id) = if let Some(ch_id) =
            quest_channel_id
        {
            // 指定されたチャンネルにメッセージを送信
            let channel_id = ChannelId::new(ch_id);
            let msg_id =
                send_quest_channel_message(&ctx.http, channel_id, guild_id, &quest_list).await?;
            (ch_id, false, Some(msg_id))
        } else {
            // チャンネルを新規作成
            let channel = discord_guild_id
                .create_channel(
                    &ctx.http,
                    CreateChannel::new("クエスト選択")
                        .kind(ChannelType::Text)
                        .category(ChannelId::new(category_id)),
                )
                .await
                .map_err(|e| {
                    error!(error = %e, guild_id, "クエストチャンネルの作成に失敗しました");
                    AppError::ChannelCreationFailed
                })?;
            let msg_id =
                send_quest_channel_message(&ctx.http, channel.id, guild_id, &quest_list).await?;
            (channel.id.get(), true, Some(msg_id))
        };

        // auto_recruitmentsテーブルに登録
        let params = CreateAutoRecruitmentParams {
            guild_id: guild_id as i64,
            category_id: category_id as i64,
            matching_channel_id: Some(final_matching_channel_id as i64),
            quest_channel_id: Some(final_quest_channel_id as i64),
            matching_channel_is_bot_created: matching_is_bot_created,
            quest_channel_is_bot_created: quest_is_bot_created,
            matching_message_id: matching_message_id.map(|id| id as i64),
            quest_message_id: quest_message_id.map(|id| id as i64),
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
/// * `command_channel_id` - コマンド実行チャンネルID
///
/// # エラー
/// * カテゴリ内のチャンネルでコマンドが実行された場合、`InCategoryChannelError`を返す
#[instrument(level = "info", skip(ctx, app_state))]
pub async fn unregister_category(
    ctx: &Context,
    app_state: &AppState,
    guild_id: u64,
    command_channel_id: u64,
) -> Result<()> {
    info!(guild_id, "自動募集カテゴリを解除します");

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

        // コマンド実行チャンネルがカテゴリ内かどうかを判定
        let command_channel = ChannelId::new(command_channel_id);
        if let Ok(channel) = command_channel.to_channel(&ctx.http).await {
            if let Some(guild_channel) = channel.guild() {
                if let Some(parent_id) = guild_channel.parent_id {
                    if parent_id.get() == auto_recruitment.category_id as u64 {
                        return Err(AppError::InCategoryChannelError);
                    }
                }
            }
        }

        // マッチングチャンネルの処理
        if let Some(matching_ch_id) = auto_recruitment.matching_channel_id {
            let channel_id = ChannelId::new(matching_ch_id as u64);
            if auto_recruitment.matching_channel_is_bot_created {
                // Bot作成チャンネルは削除
                if let Err(e) = channel_id.delete(&ctx.http).await {
                    error!(
                        channel_id = matching_ch_id,
                        error = %e,
                        "マッチングチャンネルの削除に失敗しました"
                    );
                }
            } else if let Some(msg_id) = auto_recruitment.matching_message_id {
                // 指定チャンネルはメッセージのみ削除
                let message_id = poise::serenity_prelude::MessageId::new(msg_id as u64);
                if let Err(e) = channel_id.delete_message(&ctx.http, message_id).await {
                    error!(
                        channel_id = matching_ch_id,
                        message_id = msg_id,
                        error = %e,
                        "マッチングチャンネルのメッセージ削除に失敗しました"
                    );
                }
            }
        }

        // クエストチャンネルの処理
        if let Some(quest_ch_id) = auto_recruitment.quest_channel_id {
            let channel_id = ChannelId::new(quest_ch_id as u64);
            if auto_recruitment.quest_channel_is_bot_created {
                // Bot作成チャンネルは削除
                if let Err(e) = channel_id.delete(&ctx.http).await {
                    error!(
                        channel_id = quest_ch_id,
                        error = %e,
                        "クエストチャンネルの削除に失敗しました"
                    );
                }
            } else if let Some(msg_id) = auto_recruitment.quest_message_id {
                // 指定チャンネルはメッセージのみ削除
                let message_id = poise::serenity_prelude::MessageId::new(msg_id as u64);
                if let Err(e) = channel_id.delete_message(&ctx.http, message_id).await {
                    error!(
                        channel_id = quest_ch_id,
                        message_id = msg_id,
                        error = %e,
                        "クエストチャンネルのメッセージ削除に失敗しました"
                    );
                }
            }
        }

        // 日時チャンネルの処理
        let channels = channel_repo.find_by_guild_id(&txn, guild_id as i64).await?;

        for channel in channels {
            let channel_id = ChannelId::new(channel.channel_id as u64);
            if channel.is_bot_created {
                // Bot作成チャンネルは削除
                if let Err(e) = channel_id.delete(&ctx.http).await {
                    error!(
                        channel_id = channel.channel_id,
                        error = %e,
                        "日時チャンネルの削除に失敗しました"
                    );
                }
            } else if let Some(msg_id) = channel.message_id {
                // 指定チャンネルはメッセージのみ削除
                let message_id = poise::serenity_prelude::MessageId::new(msg_id as u64);
                if let Err(e) = channel_id.delete_message(&ctx.http, message_id).await {
                    error!(
                        channel_id = channel.channel_id,
                        message_id = msg_id,
                        error = %e,
                        "日時チャンネルのメッセージ削除に失敗しました"
                    );
                }
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

                // 時間選択コンポーネントを送信してメッセージIDを取得
                let message_id = send_time_selection_message(&ctx.http, channel.id).await?;

                // DBに登録（Bot作成フラグ=true、メッセージID保存）
                channel_repo
                    .create(
                        &txn,
                        guild_id as i64,
                        channel.id.get() as i64,
                        new_date.month() as i32,
                        new_date.day() as i32,
                        sort_order,
                        true, // is_bot_created
                        Some(message_id as i64),
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
                let channel_id = ChannelId::new(channel.channel_id as u64);
                if channel.is_bot_created {
                    // Bot作成チャンネルは削除
                    if let Err(e) = channel_id.delete(&ctx.http).await {
                        error!(
                            channel_id = channel.channel_id,
                            error = %e,
                            "日時チャンネルの削除に失敗しました"
                        );
                    }
                } else if let Some(msg_id) = channel.message_id {
                    // 指定チャンネルはメッセージのみ削除
                    let message_id = poise::serenity_prelude::MessageId::new(msg_id as u64);
                    if let Err(e) = channel_id.delete_message(&ctx.http, message_id).await {
                        error!(
                            channel_id = channel.channel_id,
                            message_id = msg_id,
                            error = %e,
                            "日時チャンネルのメッセージ削除に失敗しました"
                        );
                    }
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

        // 時間選択コンポーネントを送信してメッセージIDを取得
        let message_id = send_time_selection_message(http, channel.id).await?;

        // DBに登録（Bot作成フラグ=true、メッセージID保存）
        channel_repo
            .create(
                txn,
                guild_id as i64,
                channel.id.get() as i64,
                date.month() as i32,
                date.day() as i32,
                i,
                true, // is_bot_created
                Some(message_id as i64),
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

/// 時間選択メッセージを送信し、メッセージIDを返す
///
/// グラブルではAM5:00に日付が変わるため、1/21チャンネルは「1/21 5:00〜1/22 4:00」を対象とする。
/// 選択肢は降順（夜の時間帯が先）で表示し、翌日分は「翌0:00」のように表記する。
/// 内部値は0-28（5-23は当日、24-28は翌日0-4時を表す）。
async fn send_time_selection_message(http: &Arc<Http>, channel_id: ChannelId) -> Result<u64> {
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

    // custom_id形式: auto_time_select:{channel_id}
    let custom_id = format!("auto_time_select:{}", channel_id.get());

    let select_menu = CreateSelectMenu::new(custom_id, CreateSelectMenuKind::String { options })
        .placeholder(&placeholder)
        .min_values(0)
        .max_values(24);

    let message = CreateMessage::new()
        .content("**参加可能な時間帯を選択してください**\n複数選択可能です。選択を変更すると自動的に更新されます。")
        .components(vec![CreateActionRow::SelectMenu(select_menu)]);

    let sent_message = channel_id.send_message(http, message).await.map_err(|e| {
        error!(error = %e, channel_id = channel_id.get(), "時間選択メッセージの送信に失敗しました");
        AppError::Business {
            message: "時間選択メッセージの送信に失敗しました".to_string(),
        }
    })?;

    Ok(sent_message.id.get())
}

/// マッチングチャンネルにメッセージを送信し、メッセージIDを返す
async fn send_matching_channel_message(http: &Arc<Http>, channel_id: ChannelId) -> Result<u64> {
    let message = CreateMessage::new()
        .content("**マッチング通知チャンネル**\n\n同じ日時・同じクエストを希望するユーザーが見つかると、ここに通知されます。");

    let sent_message = channel_id.send_message(http, message).await.map_err(|e| {
        error!(error = %e, channel_id = channel_id.get(), "マッチングチャンネルメッセージの送信に失敗しました");
        AppError::Business {
            message: "マッチングチャンネルメッセージの送信に失敗しました".to_string(),
        }
    })?;

    Ok(sent_message.id.get())
}

/// クエストチャンネルにメッセージを送信し、メッセージIDを返す
///
/// # 引数
/// * `http` - Discord HTTP クライアント
/// * `channel_id` - 送信先チャンネルID
/// * `guild_id` - ギルドID（カスタムID生成用）
/// * `quests` - クエストリスト (id, name)
async fn send_quest_channel_message(
    http: &Arc<Http>,
    channel_id: ChannelId,
    guild_id: u64,
    quests: &[(i32, String)],
) -> Result<u64> {
    if quests.is_empty() {
        // クエストがない場合は説明メッセージのみ
        let message = CreateMessage::new()
            .content("**クエスト選択チャンネル**\n\n現在選択可能なクエストがありません。");

        let sent_message = channel_id.send_message(http, message).await.map_err(|e| {
            error!(error = %e, channel_id = channel_id.get(), "クエストチャンネルメッセージの送信に失敗しました");
            AppError::Business {
                message: "クエストチャンネルメッセージの送信に失敗しました".to_string(),
            }
        })?;

        return Ok(sent_message.id.get());
    }

    // QuestSelectMenuBuilderを使用してセレクトメニューを構築
    let builder = QuestSelectMenuBuilder::new(guild_id).quests(quests.to_vec());

    let message = builder.build_message();

    let sent_message = channel_id.send_message(http, message).await.map_err(|e| {
        error!(error = %e, channel_id = channel_id.get(), "クエストチャンネルメッセージの送信に失敗しました");
        AppError::Business {
            message: "クエストチャンネルメッセージの送信に失敗しました".to_string(),
        }
    })?;

    Ok(sent_message.id.get())
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
