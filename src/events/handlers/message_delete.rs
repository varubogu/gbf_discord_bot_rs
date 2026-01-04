use crate::facades::recruitment::cancel::cancel_on_message_deleted;
use crate::types::{PoiseData, Result};
use poise::serenity_prelude::{ChannelId, Context, GuildId, MessageId};
use tracing::{debug, info, warn};

/// メッセージ削除イベントハンドラー
///
/// 募集メッセージが削除された場合、自動的にキャンセル処理を実行します。
pub async fn on_message_delete(
    ctx: &Context,
    channel_id: ChannelId,
    deleted_message_id: MessageId,
    guild_id: Option<GuildId>,
    data: &PoiseData,
) -> Result<()> {
    debug!(
        channel_id = %channel_id,
        message_id = %deleted_message_id,
        "メッセージ削除イベントを受信しました"
    );

    // ギルドIDの確認（DMは対象外）
    let guild_id_value = match guild_id {
        Some(id) => id.get(),
        None => {
            debug!("DMのメッセージ削除イベントのため処理をスキップします");
            return Ok(());
        }
    };

    // Facade層を呼び出して削除時キャンセル処理を実行
    match cancel_on_message_deleted(
        ctx,
        guild_id_value,
        channel_id.get(),
        deleted_message_id.get(),
        &data.app_state,
    )
    .await
    {
        Ok(true) => {
            info!(
                guild_id = %guild_id_value,
                channel_id = %channel_id,
                message_id = %deleted_message_id,
                "募集メッセージ削除に伴うキャンセル処理が完了しました"
            );
        }
        Ok(false) => {
            debug!(
                message_id = %deleted_message_id,
                "削除されたメッセージは募集メッセージではありませんでした"
            );
        }
        Err(e) => {
            // エラーが発生してもイベント処理自体は続行
            warn!(
                error = %e,
                guild_id = %guild_id_value,
                channel_id = %channel_id,
                message_id = %deleted_message_id,
                "募集キャンセル処理に失敗しました"
            );
        }
    }

    Ok(())
}
