use crate::facades::recruitment::participants::update_participants;
use crate::types::PoiseData;
use poise::serenity_prelude::{Context, Reaction};
use tracing::{info, warn};

pub async fn on_reaction_remove(
    ctx: &Context,
    reaction: &Reaction,
    data: &PoiseData,
) -> Result<(), String> {
    info!("Reaction removed:");

    // Extract required IDs from reaction
    let guild_id = reaction.guild_id.map(|id| id.get()).unwrap_or(0);
    let channel_id = reaction.channel_id.get();
    let message_id = reaction.message_id.get();

    // ボットのリアクションは無視
    if let Ok(user) = reaction.user(&ctx.http).await {
        if user.bot {
            info!("ボットのリアクションを無視します");
            return Ok(());
        }
    }

    info!(
        "参加者更新を開始します: guild: {}, channel: {}, message: {}",
        guild_id, channel_id, message_id
    );

    // Facade層を呼び出して参加者を更新
    match update_participants(ctx, guild_id, channel_id, message_id, data.app_state.db()).await {
        Ok(_) => {
            info!("参加者更新が完了しました");
            Ok(())
        }
        Err(e) => {
            warn!("参加者更新に失敗しました: {:?}", e);
            // エラーが発生してもリアクション処理自体は続行
            Ok(())
        }
    }
}
