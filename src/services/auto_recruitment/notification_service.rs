//! 自動募集通知サービス
//!
//! マッチング成功時の通知メッセージを作成・送信するサービス。
//! UIレイアウトは `NotificationPresenter` が担当し、本モジュールでは
//! プレゼンターのドメインモデルをserenityのビルダー型へ変換して送信する。

use crate::presenter::NotificationPresenter;
use crate::types::discord::{
    ActionRowContent, ComponentContent, EmbedContent, MessageContent, SelectMenuContent,
    SelectMenuKindContent,
};
use crate::types::Result;
use poise::serenity_prelude::{
    self as serenity, ChannelId, CreateActionRow, CreateEmbed, CreateMessage, CreateSelectMenu,
    CreateSelectMenuKind, CreateSelectMenuOption, EditMessage, Http, Message,
};
use std::sync::Arc;
use tracing::{debug, error, info};

/// 自動募集通知サービス
pub struct AutoRecruitmentNotificationService;

impl AutoRecruitmentNotificationService {
    pub fn new() -> Self {
        Self
    }

    /// マッチング成功通知をマッチングチャンネルに投稿
    pub async fn notify_match(
        &self,
        http: &Arc<Http>,
        channel_id: u64,
        participants: &[u64],
        quest_candidates: &[(i32, String)], // (quest_id, quest_name)
        month: i32,
        day: i32,
        hour: i32,
        matched_id: i32,
    ) -> Result<Message> {
        debug!(
            channel_id,
            participant_count = participants.len(),
            quest_count = quest_candidates.len(),
            month,
            day,
            hour,
            "マッチング成功通知を送信します"
        );

        let channel = ChannelId::new(channel_id);

        // プレゼンターでメッセージを構築
        let message_content = NotificationPresenter::create_match_notification(
            participants,
            quest_candidates,
            month,
            day,
            hour,
            matched_id,
        );
        let message = message_content_to_create_message(&message_content);

        let sent_message = channel.send_message(http, message).await.map_err(|e| {
            error!(error = %e, channel_id, "マッチング通知の送信に失敗しました");
            crate::types::AppError::Business {
                message: format!("マッチング通知の送信に失敗しました: {}", e),
            }
        })?;

        info!(
            channel_id,
            message_id = sent_message.id.get(),
            "マッチング成功通知を送信しました"
        );

        Ok(sent_message)
    }

    /// 参加者追加時にメッセージを編集
    pub async fn update_match_notification(
        &self,
        http: &Arc<Http>,
        channel_id: u64,
        message_id: u64,
        participants: &[u64],
        quest_candidates: &[(i32, String)],
        month: i32,
        day: i32,
        hour: i32,
        matched_id: i32,
    ) -> Result<Message> {
        debug!(
            channel_id,
            message_id,
            participant_count = participants.len(),
            "マッチング通知を更新します"
        );

        let channel = ChannelId::new(channel_id);

        // プレゼンターで最新のメッセージ内容を構築
        let message_content = NotificationPresenter::create_match_notification(
            participants,
            quest_candidates,
            month,
            day,
            hour,
            matched_id,
        );
        let edit_message = message_content_to_edit_message(&message_content);

        let mut message = channel
            .message(http, serenity::MessageId::new(message_id))
            .await
            .map_err(|e| {
                error!(error = %e, channel_id, message_id, "メッセージの取得に失敗しました");
                crate::types::AppError::Business {
                    message: format!("メッセージの取得に失敗しました: {}", e),
                }
            })?;

        message.edit(http, edit_message).await.map_err(|e| {
            error!(error = %e, channel_id, message_id, "メッセージの編集に失敗しました");
            crate::types::AppError::Business {
                message: format!("メッセージの編集に失敗しました: {}", e),
            }
        })?;

        info!(channel_id, message_id, "マッチング通知を更新しました");
        Ok(message)
    }

    /// 再投票メッセージを送信（元メッセージに返信）
    pub async fn send_revote_message(
        &self,
        http: &Arc<Http>,
        channel_id: u64,
        reply_to_message_id: u64,
        participants: &[u64],
        tie_quest_ids: &[(i32, String)],
        matched_id: i32,
    ) -> Result<Message> {
        debug!(
            channel_id,
            reply_to_message_id, "再投票メッセージを送信します"
        );

        let channel = ChannelId::new(channel_id);

        // プレゼンターで再投票メッセージを構築
        let message_content =
            NotificationPresenter::create_revote_notification(participants, tie_quest_ids, matched_id);
        let mut create_message = message_content_to_create_message(&message_content);
        create_message =
            create_message.reference_message((channel, serenity::MessageId::new(reply_to_message_id)));

        let sent_message = channel.send_message(http, create_message).await.map_err(|e| {
            error!(error = %e, channel_id, "再投票メッセージの送信に失敗しました");
            crate::types::AppError::Business {
                message: format!("再投票メッセージの送信に失敗しました: {}", e),
            }
        })?;

        info!(
            channel_id,
            message_id = sent_message.id.get(),
            "再投票メッセージを送信しました"
        );

        Ok(sent_message)
    }

    /// クエスト決定通知を送信
    pub async fn notify_quest_decided(
        &self,
        http: &Arc<Http>,
        channel_id: u64,
        participants: &[u64],
        quest_name: &str,
        month: i32,
        day: i32,
        hour: i32,
    ) -> Result<Message> {
        debug!(channel_id, quest_name, "クエスト決定通知を送信します");

        let channel = ChannelId::new(channel_id);

        // プレゼンターでクエスト決定通知を構築
        let message_content = NotificationPresenter::create_quest_decided_notification(
            participants,
            quest_name,
            month,
            day,
            hour,
        );
        let message = message_content_to_create_message(&message_content);

        let sent_message = channel.send_message(http, message).await.map_err(|e| {
            error!(error = %e, channel_id, "クエスト決定通知の送信に失敗しました");
            crate::types::AppError::Business {
                message: format!("クエスト決定通知の送信に失敗しました: {}", e),
            }
        })?;

        info!(channel_id, quest_name, "クエスト決定通知を送信しました");
        Ok(sent_message)
    }
}

impl Default for AutoRecruitmentNotificationService {
    fn default() -> Self {
        Self::new()
    }
}

/// EmbedContent を CreateEmbed に変換する（通知用の最小限フィールドのみ対応）
fn embed_content_to_create_embed(embed: &EmbedContent) -> CreateEmbed {
    let mut builder = CreateEmbed::new();

    if let Some(title) = &embed.title {
        builder = builder.title(title);
    }
    if let Some(description) = &embed.description {
        builder = builder.description(description);
    }
    if let Some(color) = embed.color {
        builder = builder.color(color);
    }

    builder
}

/// ActionRowContent を CreateActionRow に変換する（セレクトメニューのみ対応）
fn action_row_to_create_action_row(row: &ActionRowContent) -> Option<CreateActionRow> {
    if row.components.is_empty() {
        return None;
    }

    match &row.components[0] {
        ComponentContent::SelectMenu(menu) => {
            let select_menu = select_menu_to_create_select_menu(menu);
            Some(CreateActionRow::SelectMenu(select_menu))
        }
        // 現状このサービスではボタンを使用しないため、未対応コンポーネントは無視する
        ComponentContent::Button(_) => None,
    }
}

/// SelectMenuContent を CreateSelectMenu に変換する（文字列メニューのみ対応）
fn select_menu_to_create_select_menu(menu: &SelectMenuContent) -> CreateSelectMenu {
    let kind = match &menu.kind {
        SelectMenuKindContent::String { options } => {
            let serenity_options: Vec<CreateSelectMenuOption> = options
                .iter()
                .map(|opt| {
                    let mut o = CreateSelectMenuOption::new(&opt.label, &opt.value);
                    if let Some(desc) = &opt.description {
                        o = o.description(desc);
                    }
                    if let Some(emoji) = &opt.emoji {
                        o = o.emoji(serenity::ReactionType::Unicode(emoji.clone()));
                    }
                    if opt.default {
                        o = o.default_selection(true);
                    }
                    o
                })
                .collect();
            CreateSelectMenuKind::String {
                options: serenity_options,
            }
        }
        // 通知サービスではその他の種別は使用しない想定
        _ => CreateSelectMenuKind::String {
            options: Vec::new(),
        },
    };

    let mut select_menu = CreateSelectMenu::new(&menu.custom_id, kind);

    if let Some(placeholder) = &menu.placeholder {
        select_menu = select_menu.placeholder(placeholder);
    }
    if let Some(min) = menu.min_values {
        select_menu = select_menu.min_values(min);
    }
    if let Some(max) = menu.max_values {
        select_menu = select_menu.max_values(max);
    }
    if menu.disabled {
        select_menu = select_menu.disabled(true);
    }

    select_menu
}

/// MessageContent を CreateMessage に変換する
fn message_content_to_create_message(message: &MessageContent) -> CreateMessage {
    let mut create_message = CreateMessage::new();

    if let Some(text) = &message.text {
        create_message = create_message.content(text);
    }

    for embed in &message.embeds {
        create_message = create_message.embed(embed_content_to_create_embed(embed));
    }

    let action_rows: Vec<CreateActionRow> = message
        .components
        .iter()
        .filter_map(action_row_to_create_action_row)
        .collect();

    if !action_rows.is_empty() {
        create_message = create_message.components(action_rows);
    }

    create_message
}

/// MessageContent を EditMessage に変換する
fn message_content_to_edit_message(message: &MessageContent) -> EditMessage {
    let mut edit_message = EditMessage::new();

    if let Some(text) = &message.text {
        edit_message = edit_message.content(text);
    }

    for embed in &message.embeds {
        edit_message = edit_message.embed(embed_content_to_create_embed(embed));
    }

    let action_rows: Vec<CreateActionRow> = message
        .components
        .iter()
        .filter_map(action_row_to_create_action_row)
        .collect();

    if !action_rows.is_empty() {
        edit_message = edit_message.components(action_rows);
    }

    edit_message
}
