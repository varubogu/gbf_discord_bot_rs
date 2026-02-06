//! 自動募集カテゴリ登録コマンド

use crate::events::permission::check_bot_control_role;
use crate::facades::auto_recruitment;
use crate::gateway::PoiseDiscordGateway;
use crate::services::message::MessageTextId;
use crate::types::{AppError, PoiseContext, Result};
use poise::serenity_prelude::Channel;
use rust_i18n::t;
use tracing::error;

/// 自動募集カテゴリを登録
///
/// カテゴリを自動募集用として登録し、日時チャンネルを作成します。
#[poise::command(
    slash_command,
    guild_only,
    check = "check_bot_control_role",
    ephemeral = true,
    rename = "auto_recruit_category_register",
    name_localized("ja", "自動募集カテゴリ登録"),
    description_localized(
        "ja",
        "カテゴリを自動募集用として登録します（gbf_bot_controlロール必須）"
    )
)]
pub async fn auto_recruit_category_register(
    ctx: PoiseContext<'_>,

    #[name_localized("ja", "カテゴリ")]
    #[description = "Category channel"]
    #[description_localized("ja", "自動募集用のカテゴリチャンネル")]
    category: Channel,

    #[name_localized("ja", "募集日数")]
    #[description = "Days range (2-7, default: 7)"]
    #[description_localized("ja", "募集する日数（2〜7日、デフォルト: 7日）")]
    #[min = 2]
    #[max = 7]
    days: Option<i32>,

    #[name_localized("ja", "マッチングチャンネル")]
    #[description = "Matching notification channel"]
    #[description_localized("ja", "マッチング成功時の通知チャンネル（省略可）")]
    matching_channel: Option<Channel>,

    #[name_localized("ja", "クエストチャンネル")]
    #[description = "Quest selection channel"]
    #[description_localized("ja", "クエスト選択用のチャンネル（省略可）")]
    quest_channel: Option<Channel>,
) -> Result<()> {
    ctx.defer_ephemeral().await?;

    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| crate::types::AppError::Business {
            message: "このコマンドはサーバー内でのみ使用できます。".to_string(),
        })?;

    // カテゴリチャンネルかどうかを確認
    let category_id = category.id().get();
    if !matches!(
        category,
        Channel::Guild(ref gc) if gc.kind == poise::serenity_prelude::ChannelType::Category
    ) {
        return Err(crate::types::AppError::Business {
            message: "カテゴリチャンネルを指定してください。".to_string(),
        });
    }

    let matching_channel_id = matching_channel.map(|c| c.id().get());
    let quest_channel_id = quest_channel.map(|c| c.id().get());
    let days_range = days.unwrap_or(7);

    let app_state = &ctx.data().app_state;
    let gateway = PoiseDiscordGateway::new(std::sync::Arc::clone(&ctx.serenity_context().http));

    match auto_recruitment::register_category(
        &gateway,
        app_state,
        guild_id.get(),
        category_id,
        days_range,
        matching_channel_id,
        quest_channel_id,
    )
    .await
    {
        Ok(result) => {
            let mut message = format!(
                "✅ 自動募集カテゴリを登録しました。\n\n\
                 **カテゴリ:** <#{}>\n\
                 **募集日数:** {}日\n\
                 **作成されたチャンネル数:** {}",
                result.category_id, days_range, result.channel_count
            );

            if let Some(ch_id) = matching_channel_id {
                message.push_str(&format!("\n**マッチングチャンネル:** <#{ch_id}>"));
            }
            if let Some(ch_id) = quest_channel_id {
                message.push_str(&format!("\n**クエストチャンネル:** <#{ch_id}>"));
            }

            ctx.send(
                poise::CreateReply::default()
                    .content(message)
                    .ephemeral(true),
            )
            .await?;
        }
        Err(e) => {
            error!(error = %e, guild_id = guild_id.get(), "自動募集カテゴリの登録に失敗しました");

            // エラーメッセージを多言語対応で取得
            let error_message = match &e {
                AppError::ChannelCreationFailed => {
                    let locale = ctx.locale().unwrap_or("ja");
                    t!(
                        MessageTextId::AutoRecruitmentChannelCreateFailed.as_str(),
                        locale = locale
                    )
                    .to_string()
                }
                _ => format!("エラー: {e}"),
            };

            ctx.send(
                poise::CreateReply::default()
                    .content(error_message)
                    .ephemeral(true),
            )
            .await?;
        }
    }

    Ok(())
}
