use sea_orm::TransactionTrait;

use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::repository::database::channel_type_repository::ChannelTypeRepository;
use crate::repository::database::guild_channel_repository::GuildChannelRepository;
use crate::types::{PoiseContext, Result};

/// チャンネル設定を表示
///
/// ギルドの通知チャンネル設定を表示します。
#[poise::command(
    slash_command,
    guild_only,
    ephemeral = true,
    rename = "channel_show",
    name_localized("ja", "チャンネル設定表示"),
    description_localized("ja", "ギルドの通知チャンネル設定を表示します。"),
)]
pub async fn channel_show(ctx: PoiseContext<'_>) -> Result<()> {
    ctx.defer_ephemeral().await?;

    let guild_id = ctx.guild_id().ok_or_else(|| {
        crate::types::AppError::Business {
            message: "このコマンドはサーバー内でのみ使用できます。".to_string(),
        }
    })?;

    let app_state = &ctx.data().app_state;
    let txn = app_state.guild_db().begin().await?;

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id.get() as i64).await?;

    let result = async {
        let channel_type_repo = ChannelTypeRepository::new();
        let guild_channel_repo = GuildChannelRepository::new();

        // 全チャンネル種別を取得
        let all_channel_types = channel_type_repo.get_all(&txn).await?;

        if all_channel_types.is_empty() {
            ctx.send(
                poise::CreateReply::default()
                    .content("⚠️ チャンネル種別が登録されていません。")
                    .ephemeral(true),
            )
            .await?;
            return Ok::<(), crate::types::AppError>(());
        }

        // ギルドのチャンネル設定を取得
        let guild_channels = guild_channel_repo
            .get_all_by_guild_with_txn(&txn, guild_id.get() as i64)
            .await?;

        // チャンネルIDでマップを作成
        let channel_map: std::collections::HashMap<i32, i64> = guild_channels
            .iter()
            .map(|gc| (gc.channel_type, gc.channel_id))
            .collect();

        // トランザクションをコミット
        txn.commit().await?;

        // 設定状況を作成
        let mut status_message = "**現在のチャンネル設定:**\n\n".to_string();

        for ct in all_channel_types {
            if let Some(channel_id) = channel_map.get(&ct.id) {
                status_message.push_str(&format!("• **{}**: <#{}>\n", ct.name, channel_id));
            } else {
                status_message.push_str(&format!("• **{}**: 未設定\n", ct.name));
            }
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
