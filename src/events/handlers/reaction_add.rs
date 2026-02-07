use crate::facades::recruitment::participants::update_participants;
use crate::gateway::PoiseDiscordGateway;
use crate::types::PoiseData;
use poise::serenity_prelude::{Context, Reaction};
use std::sync::Arc;
use tracing::{debug, info, warn};

pub async fn on_reaction_add(
    ctx: &Context,
    reaction: &Reaction,
    data: &PoiseData,
) -> Result<(), String> {
    debug!("Reaction added:");

    // Extract required IDs from reaction
    let guild_id = match reaction.guild_id {
        Some(id) => id.get(),
        None => {
            warn!("ギルドIDが取得できませんでした");
            return Err("ギルドIDが取得できませんでした".to_string());
        }
    };
    let channel_id = reaction.channel_id.get();
    let message_id = reaction.message_id.get();

    // リアクションしたユーザーを取得（ボットのリアクションは無視）
    let user = match reaction.user(&ctx.http).await {
        Ok(user) => {
            if user.bot {
                debug!("ボットのリアクションを無視します");
                return Ok(());
            }
            user
        }
        Err(e) => {
            warn!("ユーザー取得エラー: {:?}", e);
            return Err(format!("ユーザー取得エラー: {e:?}"));
        }
    };
    let user_id = user.id;

    // リアクションの絵文字を取得（文字列に変換）
    let reaction_emoji = Some(reaction.emoji.to_string());

    info!(
        "参加者更新を開始します: guild: {}, channel: {}, message: {}, user: {}",
        guild_id, channel_id, message_id, user_id
    );

    // events層でGatewayを作成
    let gateway = PoiseDiscordGateway::new(Arc::clone(&ctx.http));

    // Facade層を呼び出して参加者を更新（DB登録含む）
    match update_participants(
        &gateway,
        guild_id,
        channel_id,
        message_id,
        Some(user_id.get()),
        reaction_emoji,
        &data.app_state,
    )
    .await
    {
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
