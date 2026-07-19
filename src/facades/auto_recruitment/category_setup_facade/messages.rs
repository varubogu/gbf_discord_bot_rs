use super::common::localized_ja;
use crate::gateway::DiscordMessageGateway;
use crate::models::quests::Quest;
use crate::presenter::auto_recruitment_presenter::AutoRecruitmentPresenter;
use crate::services::message::MessageTextId;
use crate::types::discord::{
    ActionRowContent, ButtonContent, ButtonStyleType, DiscordChannelId, DiscordMessageId,
    MessageContent, SelectMenuContent, SelectMenuOptionContent,
};
use crate::types::{AppError, BattleStyleId, Result};
use tracing::{debug, error, info};

/// 時間選択メッセージを送信し、メッセージIDを返す
///
/// グラブルではAM5:00に日付が変わるため、1/21チャンネルは「1/21 5:00〜1/22 4:00」を対象とする。
/// 選択肢は降順（夜の時間帯が先）で表示し、翌日分は「翌0:00」のように表記する。
/// 内部値は0-28（5-23は当日、24-28は翌日0-4時を表す）。
pub(super) async fn send_time_selection_message<G>(
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
pub(super) async fn send_matching_channel_message<G>(
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
pub(super) async fn send_quest_channel_messages<G>(
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
