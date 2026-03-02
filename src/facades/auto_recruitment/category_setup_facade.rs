//! 自動募集カテゴリ設定Facade
//!
//! カテゴリ登録/解除/日数変更の処理を行う

use crate::gateway::{DiscordChannelGateway, DiscordMessageGateway};
use crate::infrastructure::database::session::set_current_guild_id;
use crate::models::quests::Quest;
use crate::presenter::auto_recruitment_presenter::AutoRecruitmentPresenter;
use crate::services::auto_recruitment::CategorySetupService;
use crate::services::message::MessageTextId;
use crate::types::discord::{
    ActionRowContent, ButtonContent, ButtonStyleType, ChannelCreateParams, ChannelEditParams,
    DiscordChannelId, DiscordGuildId, DiscordMessageId, MessageContent, SelectMenuContent,
    SelectMenuOptionContent,
};
use crate::types::{AppError, AppState, BattleStyleId, Result};
use chrono::{Datelike, Duration, Utc};
use rust_i18n::t;
use sea_orm::TransactionTrait;
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
/// * `gateway` - Discord Gateway
/// * `app_state` - アプリケーション状態
/// * `guild_id` - ギルドID
/// * `category_id` - カテゴリチャンネルID
/// * `days` - 募集日数（2-7日）
/// * `matching_channel_id` - マッチング通知チャンネルID（省略可能）
/// * `quest_channel_id` - クエスト選択チャンネルID（省略可能）
#[instrument(level = "info", skip(gateway, app_state))]
pub async fn register_category<G>(
    gateway: &G,
    app_state: &AppState,
    guild_id: u64,
    category_id: u64,
    days: i32,
    matching_channel_id: Option<u64>,
    quest_channel_id: Option<u64>,
) -> Result<CategoryRegistrationResult>
where
    G: DiscordChannelGateway + DiscordMessageGateway + Sync,
{
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
        let setup_service = CategorySetupService::new(
            app_state.repositories.auto_recruitment,
            app_state.repositories.auto_recruitment_channel,
            app_state.repositories.quest,
            app_state.repositories.auto_recruitment_quest_message,
            app_state.repositories.quest_matching_user,
            app_state.repositories.quest_matching,
            app_state.repositories.scheduled_task,
        );

        // 既存の登録をチェック
        setup_service
            .ensure_not_registered(&txn, guild_id as i64)
            .await?;

        let discord_guild_id = DiscordGuildId::new(guild_id);

        // チャンネル順序: マッチング(0) → 日付昇順(1〜days) → クエスト(days+1)

        // マッチングチャンネルの処理（position 0）
        let (final_matching_channel_id, matching_is_bot_created, matching_message_id) =
            if let Some(ch_id) = matching_channel_id {
                // 指定されたチャンネルにメッセージを送信し、位置を調整
                let channel_id = DiscordChannelId::new(ch_id);
                let msg_id = send_matching_channel_message(gateway, channel_id).await?;
                // 指定チャンネルの位置を0に設定
                let _ = gateway
                    .edit_channel(channel_id, ChannelEditParams::new().with_position(0))
                    .await;
                (ch_id, false, Some(msg_id.get()))
            } else {
                // チャンネルを新規作成（position 0）
                let channel_params = ChannelCreateParams::text("マッチング")
                    .with_parent(DiscordChannelId::new(category_id))
                    .with_position(0);

                let new_channel_id = gateway
                    .create_channel(discord_guild_id, channel_params)
                    .await
                    .map_err(|e| {
                        error!(error = %e, guild_id, "マッチングチャンネルの作成に失敗しました");
                        AppError::ChannelCreationFailed
                    })?;

                let msg_id = send_matching_channel_message(gateway, new_channel_id).await?;
                (new_channel_id.get(), true, Some(msg_id.get()))
            };

        // クエスト一覧を取得（有効なクエストのみ）
        let enabled_quests = setup_service
            .get_enabled_quests(&txn, guild_id as i64)
            .await?;

        // クエストチャンネルの処理（position days+1、日付チャンネル作成後に位置を設定）
        let (final_quest_channel_id, quest_is_bot_created) = if let Some(ch_id) = quest_channel_id {
            (ch_id, false)
        } else {
            // チャンネルを新規作成（位置は後で設定）
            let channel_params = ChannelCreateParams::text("クエスト選択")
                .with_parent(DiscordChannelId::new(category_id));

            let new_channel_id = gateway
                .create_channel(discord_guild_id, channel_params)
                .await
                .map_err(|e| {
                    error!(error = %e, guild_id, "クエストチャンネルの作成に失敗しました");
                    AppError::ChannelCreationFailed
                })?;

            (new_channel_id.get(), true)
        };

        // 1クエスト1メッセージ形式でメッセージを送信し、メッセージIDを保存
        let quest_channel_id_domain = DiscordChannelId::new(final_quest_channel_id);
        let quest_message_mappings = send_quest_channel_messages(
            gateway,
            quest_channel_id_domain,
            guild_id,
            &enabled_quests,
        )
        .await?;
        for (quest_id, sent_message_id) in quest_message_mappings {
            setup_service
                .upsert_quest_message(&txn, guild_id as i64, quest_id, sent_message_id)
                .await?;
        }

        // auto_recruitmentsテーブルに登録
        let auto_recruitment = setup_service
            .create_auto_recruitment(
                &txn,
                guild_id as i64,
                category_id as i64,
                Some(final_matching_channel_id as i64),
                Some(final_quest_channel_id as i64),
                matching_is_bot_created,
                quest_is_bot_created,
                matching_message_id.map(|id| id as i64),
                days,
            )
            .await?;

        info!(
            guild_id = auto_recruitment.guild_id,
            "自動募集設定を登録しました"
        );

        // 日時チャンネルを作成（position 1〜days）
        let now_utc = Utc::now();
        let now_jst = now_utc + Duration::hours(9);
        let today = now_jst.date_naive();
        let discord_guild_id = DiscordGuildId::new(guild_id);
        let mut created_channels = 0;

        for i in 0..days {
            let date = today + Duration::days(i as i64);
            let channel_name = format!("{}月{}日", date.month(), date.day());
            let channel_position = (i + 1) as u16;

            let channel_params = ChannelCreateParams::text(&channel_name)
                .with_parent(DiscordChannelId::new(category_id))
                .with_position(channel_position);

            let new_channel_id = gateway
                .create_channel(discord_guild_id, channel_params)
                .await
                .map_err(|e| {
                    error!(error = %e, guild_id, category_id, "チャンネルの作成に失敗しました");
                    AppError::ChannelCreationFailed
                })?;

            let message_id = send_time_selection_message(gateway, new_channel_id).await?;

            setup_service
                .create_date_channel(
                    &txn,
                    guild_id as i64,
                    new_channel_id.get() as i64,
                    date.month() as i32,
                    date.day() as i32,
                    i,
                    true,
                    Some(message_id.get() as i64),
                )
                .await?;

            created_channels += 1;
            debug!(
                channel_id = new_channel_id.get(),
                channel_name, "日時チャンネルを作成しました"
            );
        }

        // クエストチャンネルの位置を日付チャンネルの後に設定（position days+1）
        let quest_channel = DiscordChannelId::new(final_quest_channel_id);
        let _ = gateway
            .edit_channel(
                quest_channel,
                ChannelEditParams::new().with_position((days + 1) as u16),
            )
            .await;

        // ローテーションタスクと自動マッチングタスクを初期登録
        setup_service.ensure_initial_rotation_task(&txn).await?;
        setup_service
            .ensure_initial_auto_matching_task(&txn)
            .await?;

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
/// * `gateway` - Discord Gateway
/// * `app_state` - アプリケーション状態
/// * `guild_id` - ギルドID
/// * `command_channel_id` - コマンド実行チャンネルID
///
/// # エラー
/// * カテゴリ内のチャンネルでコマンドが実行された場合、`InCategoryChannelError`を返す
#[instrument(level = "info", skip(gateway, app_state))]
pub async fn unregister_category<G>(
    gateway: &G,
    app_state: &AppState,
    guild_id: u64,
    command_channel_id: u64,
) -> Result<()>
where
    G: DiscordChannelGateway + DiscordMessageGateway + Sync,
{
    info!(guild_id, "自動募集カテゴリを解除します");

    let conn = app_state.guild_db();
    let txn = conn.begin().await?;

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id as i64).await?;

    let result = async {
        let setup_service = CategorySetupService::new(
            app_state.repositories.auto_recruitment,
            app_state.repositories.auto_recruitment_channel,
            app_state.repositories.quest,
            app_state.repositories.auto_recruitment_quest_message,
            app_state.repositories.quest_matching_user,
            app_state.repositories.quest_matching,
            app_state.repositories.scheduled_task,
        );

        // 自動募集設定を取得
        let auto_recruitment = setup_service
            .get_auto_recruitment_or_err(&txn, guild_id as i64)
            .await?;

        // コマンド実行チャンネルがカテゴリ内かどうかを判定
        let command_channel = DiscordChannelId::new(command_channel_id);
        if let Ok(channel_data) = gateway.get_channel(command_channel).await
            && let Some(parent_id) = channel_data.parent_id
            && parent_id.get() == auto_recruitment.category_id as u64
        {
            return Err(AppError::InCategoryChannelError);
        }

        // マッチングチャンネルの処理
        if let Some(matching_ch_id) = auto_recruitment.matching_channel_id {
            let channel_id = DiscordChannelId::new(matching_ch_id as u64);
            if auto_recruitment.matching_channel_is_bot_created {
                // Bot作成チャンネルは削除
                if let Err(e) = gateway.delete_channel(channel_id).await {
                    error!(
                        channel_id = matching_ch_id,
                        error = %e,
                        "マッチングチャンネルの削除に失敗しました"
                    );
                }
            } else if let Some(msg_id) = auto_recruitment.matching_message_id {
                // 指定チャンネルはメッセージのみ削除
                let message_id = DiscordMessageId::new(msg_id as u64);
                if let Err(e) = gateway.delete_message(channel_id, message_id).await {
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
            let channel_id = DiscordChannelId::new(quest_ch_id as u64);
            if auto_recruitment.quest_channel_is_bot_created {
                // Bot作成チャンネルは削除
                if let Err(e) = gateway.delete_channel(channel_id).await {
                    error!(
                        channel_id = quest_ch_id,
                        error = %e,
                        "クエストチャンネルの削除に失敗しました"
                    );
                }
            } else {
                // 指定チャンネルは各クエストメッセージを削除
                let quest_messages = setup_service
                    .find_quest_messages(&txn, guild_id as i64)
                    .await?;

                for quest_msg in quest_messages {
                    let message_id = DiscordMessageId::new(quest_msg.message_id as u64);
                    if let Err(e) = gateway.delete_message(channel_id, message_id).await {
                        error!(
                            channel_id = quest_ch_id,
                            message_id = quest_msg.message_id,
                            quest_id = quest_msg.quest_id,
                            error = %e,
                            "クエストメッセージの削除に失敗しました"
                        );
                    }
                }
            }
        }

        // クエストメッセージのDBレコードを削除
        setup_service
            .delete_all_quest_messages(&txn, guild_id as i64)
            .await?;

        // 日時チャンネルの処理
        let channels = setup_service
            .find_date_channels(&txn, guild_id as i64)
            .await?;

        for channel in channels {
            let channel_id = DiscordChannelId::new(channel.channel_id as u64);
            if channel.is_bot_created {
                // Bot作成チャンネルは削除
                if let Err(e) = gateway.delete_channel(channel_id).await {
                    error!(
                        channel_id = channel.channel_id,
                        error = %e,
                        "日時チャンネルの削除に失敗しました"
                    );
                }
            } else if let Some(msg_id) = channel.message_id {
                // 指定チャンネルはメッセージのみ削除
                let message_id = DiscordMessageId::new(msg_id as u64);
                if let Err(e) = gateway.delete_message(channel_id, message_id).await {
                    error!(
                        channel_id = channel.channel_id,
                        message_id = msg_id,
                        error = %e,
                        "日時チャンネルのメッセージ削除に失敗しました"
                    );
                }
            }
        }

        // マッチング関連データを削除（外部キー制約順）
        setup_service
            .delete_all_matching_data(&txn, guild_id as i64)
            .await?;

        // DBから削除
        setup_service
            .delete_all_date_channels(&txn, guild_id as i64)
            .await?;
        setup_service
            .delete_auto_recruitment(&txn, guild_id as i64)
            .await?;

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
/// * `gateway` - Discord Gateway
/// * `app_state` - アプリケーション状態
/// * `guild_id` - ギルドID
/// * `new_days` - 新しい募集日数（2-7日）
#[instrument(level = "info", skip(gateway, app_state))]
pub async fn change_days<G>(
    gateway: &G,
    app_state: &AppState,
    guild_id: u64,
    new_days: i32,
) -> Result<()>
where
    G: DiscordChannelGateway + DiscordMessageGateway + Sync,
{
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
        let setup_service = CategorySetupService::new(
            app_state.repositories.auto_recruitment,
            app_state.repositories.auto_recruitment_channel,
            app_state.repositories.quest,
            app_state.repositories.auto_recruitment_quest_message,
            app_state.repositories.quest_matching_user,
            app_state.repositories.quest_matching,
            app_state.repositories.scheduled_task,
        );

        // 自動募集設定を取得
        let auto_recruitment = setup_service
            .get_auto_recruitment_or_err(&txn, guild_id as i64)
            .await?;

        let current_days = auto_recruitment.days_range;

        if new_days == current_days {
            return Err(AppError::Business {
                message: format!("募集日数は既に{new_days}日です"),
            });
        }

        // 既存のチャンネルを取得
        let existing_channels = setup_service
            .find_date_channels(&txn, guild_id as i64)
            .await?;

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

            let discord_guild_id = DiscordGuildId::new(guild_id);

            for i in 1..=channels_to_add {
                let new_date = last_channel_date + Duration::days(i as i64);
                let channel_name = format!("{}月{}日", new_date.month(), new_date.day());
                let sort_order = (existing_channels.len() + i) as i32;
                // 日付チャンネルはposition 1から（position 0はマッチング）
                let channel_position = (existing_channels.len() + i) as u16;

                // Discordチャンネルを作成（カテゴリの権限を継承、位置指定）
                let channel_params = ChannelCreateParams::text(&channel_name)
                    .with_parent(DiscordChannelId::new(category_id))
                    .with_position(channel_position);

                let new_channel_id = gateway
                    .create_channel(discord_guild_id, channel_params)
                    .await
                    .map_err(|e| {
                        error!(error = %e, guild_id, category_id, "チャンネルの作成に失敗しました");
                        AppError::ChannelCreationFailed
                    })?;

                // 時間選択コンポーネントを送信してメッセージIDを取得
                let message_id = send_time_selection_message(gateway, new_channel_id).await?;

                // DBに登録（Bot作成フラグ=true、メッセージID保存）
                setup_service
                    .create_date_channel(
                        &txn,
                        guild_id as i64,
                        new_channel_id.get() as i64,
                        new_date.month() as i32,
                        new_date.day() as i32,
                        sort_order,
                        true,
                        Some(message_id.get() as i64),
                    )
                    .await?;
            }

            // クエストチャンネルの位置を日付チャンネルの後に更新
            if let Some(quest_ch_id) = auto_recruitment.quest_channel_id {
                let quest_channel = DiscordChannelId::new(quest_ch_id as u64);
                let _ = gateway
                    .edit_channel(
                        quest_channel,
                        ChannelEditParams::new().with_position((new_days + 1) as u16),
                    )
                    .await;
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
                let channel_id = DiscordChannelId::new(channel.channel_id as u64);
                if channel.is_bot_created {
                    // Bot作成チャンネルは削除
                    if let Err(e) = gateway.delete_channel(channel_id).await {
                        error!(
                            channel_id = channel.channel_id,
                            error = %e,
                            "日時チャンネルの削除に失敗しました"
                        );
                    }
                } else if let Some(msg_id) = channel.message_id {
                    // 指定チャンネルはメッセージのみ削除
                    let message_id = DiscordMessageId::new(msg_id as u64);
                    if let Err(e) = gateway.delete_message(channel_id, message_id).await {
                        error!(
                            channel_id = channel.channel_id,
                            message_id = msg_id,
                            error = %e,
                            "日時チャンネルのメッセージ削除に失敗しました"
                        );
                    }
                }

                // DBから削除
                setup_service
                    .delete_date_channel_by_channel_id(&txn, guild_id as i64, channel.channel_id)
                    .await?;
            }
        }

        // 日数を更新
        setup_service
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

/// 時間選択メッセージを送信し、メッセージIDを返す
///
/// グラブルではAM5:00に日付が変わるため、1/21チャンネルは「1/21 5:00〜1/22 4:00」を対象とする。
/// 選択肢は降順（夜の時間帯が先）で表示し、翌日分は「翌0:00」のように表記する。
/// 内部値は0-28（5-23は当日、24-28は翌日0-4時を表す）。
async fn send_time_selection_message<G>(
    gateway: &G,
    channel_id: DiscordChannelId,
) -> Result<DiscordMessageId>
where
    G: DiscordMessageGateway + Sync,
{
    // ゲーム内日付: 当日5:00〜翌日4:00（24時間）
    // 降順で表示: 翌4:00, 翌3:00, 翌2:00, 翌1:00, 翌0:00, 23:00, 22:00, ..., 5:00
    let mut options: Vec<SelectMenuOptionContent> = Vec::with_capacity(24);

    // 翌日分（4:00→0:00の降順）- 内部値は28, 27, 26, 25, 24
    for hour in (0..=4).rev() {
        let label = format!("翌{hour}:00");
        let value = (24 + hour).to_string(); // 24-28
        options.push(SelectMenuOptionContent::new(label, value));
    }

    // 当日分（23:00→5:00の降順）- 内部値は23, 22, ..., 5
    for hour in (5..=23).rev() {
        let label = format!("{hour}:00");
        let value = hour.to_string();
        options.push(SelectMenuOptionContent::new(label, value));
    }

    // 多言語対応のplaceholderを取得（デフォルトは日本語）
    let placeholder = localized_ja(MessageTextId::AutoRecruitmentTimeSelectPlaceholder);

    // custom_id形式: auto_time_select:{channel_id}
    let custom_id = format!("auto_time_select:{}", channel_id.get());

    // ドメインモデルでセレクトメニューを作成
    let select_menu = SelectMenuContent::string_select(&custom_id, options)
        .with_placeholder(&placeholder)
        .with_min_values(0)
        .with_max_values(24);

    let action_row = ActionRowContent::select_menu(select_menu);

    // ドメインモデルでメッセージを作成
    let message_content = MessageContent::new()
        .with_text(localized_ja(
            MessageTextId::AutoRecruitmentCategorySetupTimeSelectMessage,
        ))
        .with_component(action_row);

    let sent_message_id = gateway
        .send_message(channel_id, message_content)
        .await
        .map_err(|e| {
            error!(error = %e, channel_id = channel_id.get(), "時間選択メッセージの送信に失敗しました");
            AppError::Business {
                message: "時間選択メッセージの送信に失敗しました".to_string(),
            }
        })?;

    Ok(sent_message_id)
}

/// マッチングチャンネルにメッセージを送信し、メッセージIDを返す
async fn send_matching_channel_message<G>(
    gateway: &G,
    channel_id: DiscordChannelId,
) -> Result<DiscordMessageId>
where
    G: DiscordMessageGateway + Sync,
{
    // ドメインモデルでメッセージを作成
    let message_content = MessageContent::text(localized_ja(
        MessageTextId::AutoRecruitmentCategorySetupMatchingChannelMessage,
    ));

    let sent_message_id = gateway
        .send_message(channel_id, message_content)
        .await
        .map_err(|e| {
            error!(error = %e, channel_id = channel_id.get(), "マッチングチャンネルメッセージの送信に失敗しました");
            AppError::Business {
                message: "マッチングチャンネルメッセージの送信に失敗しました".to_string(),
            }
        })?;

    Ok(sent_message_id)
}

/// クエストチャンネルに1クエスト1メッセージ形式でメッセージを送信
///
/// # 引数
/// * `gateway` - Discord Gateway
/// * `channel_id` - 送信先チャンネルID
/// * `guild_id` - ギルドID（カスタムID生成用）
/// * `quests` - クエストリスト（available_battle_style_ids含む）
///
/// # 戻り値
/// 送信したクエストメッセージの `(quest_id, message_id)` 一覧
async fn send_quest_channel_messages<G>(
    gateway: &G,
    channel_id: DiscordChannelId,
    guild_id: u64,
    quests: &[Quest],
) -> Result<Vec<(i32, i64)>>
where
    G: DiscordMessageGateway + Sync,
{
    if quests.is_empty() {
        // クエストがない場合は説明メッセージのみ
        let message_content = MessageContent::text(localized_ja(
            MessageTextId::AutoRecruitmentCategorySetupQuestChannelEmptyMessage,
        ));

        gateway.send_message(channel_id, message_content).await.map_err(|e| {
            error!(error = %e, channel_id = channel_id.get(), "クエストチャンネルメッセージの送信に失敗しました");
            AppError::Business {
                message: "クエストチャンネルメッセージの送信に失敗しました".to_string(),
            }
        })?;

        return Ok(Vec::new());
    }

    let mut quest_message_mappings = Vec::with_capacity(quests.len());

    // 各クエストに対してメッセージを送信
    for quest in quests {
        // AutoRecruitmentPresenterを使用してメッセージを構築
        // default_battle_style_idで6属性クエストかどうかを判定
        let is_six_element = BattleStyleId::is_six_elements(quest.default_battle_style_id);
        let message_content = AutoRecruitmentPresenter::create_quest_message(
            guild_id,
            quest.id,
            &quest.name,
            is_six_element,
        );

        let sent_message_id = gateway.send_message(channel_id, message_content).await.map_err(|e| {
            error!(error = %e, channel_id = channel_id.get(), quest_id = quest.id, "クエストメッセージの送信に失敗しました");
            AppError::Business {
                message: format!("クエストメッセージの送信に失敗しました: {}", quest.name),
            }
        })?;

        quest_message_mappings.push((quest.id, sent_message_id.get() as i64));

        debug!(
            quest_id = quest.id,
            message_id = sent_message_id.get(),
            "クエストメッセージを送信しました"
        );
    }

    // 最後に「選択済みのクエスト」ボタン付きメッセージを送信（ドメインモデル使用）
    let check_button = ButtonContent::new(
        format!("auto_quest_selection_check:{guild_id}"),
        localized_ja(MessageTextId::AutoRecruitmentCategorySetupSelectionCheckButton),
    )
    .with_style(ButtonStyleType::Secondary);

    let action_row = ActionRowContent::buttons(vec![check_button]);

    let check_message_content = MessageContent::new()
        .with_text(localized_ja(
            MessageTextId::AutoRecruitmentCategorySetupSelectionCheckMessage,
        ))
        .with_component(action_row);

    gateway
        .send_message(channel_id, check_message_content)
        .await
        .map_err(|e| {
            error!(error = %e, channel_id = channel_id.get(), "選択確認メッセージの送信に失敗しました");
            AppError::Business {
                message: "選択確認メッセージの送信に失敗しました".to_string(),
            }
        })?;

    info!(
        guild_id,
        quest_count = quests.len(),
        "クエストメッセージを全て送信しました"
    );

    Ok(quest_message_mappings)
}

/// 自動募集関連メッセージを日本語で取得する
fn localized_ja(message_id: MessageTextId) -> String {
    t!(message_id.as_str(), locale = "ja").to_string()
}
