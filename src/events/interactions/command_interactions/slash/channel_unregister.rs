use poise::serenity_prelude::AutocompleteChoice;
use sea_orm::TransactionTrait;
use tracing::{error, info};

use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::repository::database::channel_type_repository::ChannelTypeRepository;
use crate::repository::database::guild_channel_repository::GuildChannelRepository;
use crate::services::permission::check_bot_control_role;
use crate::types::{PoiseContext, Result};

/// チャンネル種別の選択肢を取得
async fn channel_type_autocomplete<'a>(
    ctx: PoiseContext<'_>,
    _partial: &'a str,
) -> impl Iterator<Item = AutocompleteChoice> + 'a {
    let db = ctx.data().app_state.guild_db();
    let channel_type_repo = ChannelTypeRepository::new();

    let channel_types = channel_type_repo
        .get_all(db)
        .await
        .unwrap_or_else(|e| {
            error!(error = %e, "チャンネル種別の取得に失敗しました");
            vec![]
        });

    channel_types
        .into_iter()
        .map(|ct| AutocompleteChoice::new(ct.name.clone(), ct.id.to_string()))
        .collect::<Vec<_>>()
        .into_iter()
}

/// チャンネルを削除
///
/// ギルドの通知チャンネル設定を削除します。
#[poise::command(
    slash_command,
    guild_only,
    check = "check_bot_control_role",
    ephemeral = true,
    rename = "channel_unregister",
    name_localized("ja", "チャンネル登録解除"),
    description_localized("ja", "ギルドの通知チャンネル設定を削除します。（gbf_bot_controlロール必須）"),
)]
pub async fn channel_unregister(
    ctx: PoiseContext<'_>,
    #[autocomplete = "channel_type_autocomplete"]
    #[name_localized("ja", "チャンネル種別")]
    #[description = "Channel type"]
    #[description_localized("ja", "削除するチャンネル種別")]
    channel_type: String,
) -> Result<()> {
    ctx.defer_ephemeral().await?;

    let guild_id = ctx.guild_id().ok_or_else(|| {
        crate::types::AppError::Business {
            message: "このコマンドはサーバー内でのみ使用できます。".to_string(),
        }
    })?;

    // channel_typeをi32に変換
    let channel_type_id: i32 = channel_type.parse().map_err(|_| {
        crate::types::AppError::Validation {
            field: "チャンネル種別".to_string(),
        }
    })?;

    info!(
        guild_id = %guild_id,
        channel_type = channel_type_id,
        "チャンネル登録解除を開始します"
    );

    let app_state = &ctx.data().app_state;
    let txn = app_state.guild_db().begin().await?;

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id.get() as i64).await?;

    let result = async {
        let channel_type_repo = ChannelTypeRepository::new();
        let guild_channel_repo = GuildChannelRepository::new();

        // チャンネル種別が存在するか確認
        let channel_type_model = channel_type_repo
            .get_by_id(&txn, channel_type_id)
            .await?
            .ok_or_else(|| crate::types::AppError::NotFound(format!(
                "チャンネル種別ID {} が見つかりませんでした",
                channel_type_id
            )))?;

        // 削除前に現在の設定を取得
        let existing_channel = guild_channel_repo
            .get_by_guild_and_type_with_txn(&txn, guild_id.get() as i64, channel_type_id)
            .await?;

        let old_channel_id = existing_channel
            .as_ref()
            .map(|c| c.channel_id)
            .ok_or_else(|| {
                crate::types::AppError::NotFound(format!(
                    "チャンネル種別「{}」の設定が見つかりませんでした",
                    channel_type_model.name
                ))
            })?;

        // ギルドチャンネルを削除
        guild_channel_repo
            .delete_with_txn(&txn, guild_id.get() as i64, channel_type_id)
            .await?;

        info!(
            guild_id = %guild_id,
            channel_type = channel_type_id,
            "チャンネル登録解除が完了しました"
        );

        // コミット前に、全チャンネル種別の設定状況を取得（トランザクション内で実行）
        let all_channel_types = channel_type_repo.get_all(&txn).await?;
        let mut status_lines = Vec::new();

        for ct in all_channel_types {
            let guild_channel = guild_channel_repo
                .get_by_guild_and_type_with_txn(&txn, guild_id.get() as i64, ct.id)
                .await?;

            if let Some(gc) = guild_channel {
                status_lines.push(format!("• **{}**: <#{}>\n", ct.name, gc.channel_id));
            } else {
                status_lines.push(format!("• **{}**: 未設定\n", ct.name));
            }
        }

        // トランザクションをコミット（ここで確定させる）
        txn.commit().await?;

        // 削除後、設定状況を表示
        let mut status_message = format!(
            "✅ チャンネル設定を削除しました。\n\n**種別:** {}\n**削除されたチャンネル:** <#{}>\n\n**現在の設定状況:**\n",
            channel_type_model.name, old_channel_id
        );

        for line in status_lines {
            status_message.push_str(&line);
        }

        ctx.send(
            poise::CreateReply::default()
                .content(status_message)
                .ephemeral(true),
        )
        .await?;

        Ok::<(), crate::types::AppError>(())
    }
    .await;

    result
}
