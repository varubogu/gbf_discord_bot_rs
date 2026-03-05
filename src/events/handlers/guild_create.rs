use crate::events::init_message::build_init_guide_message;
use crate::facades::guild::guild_management_facade::GuildManagementFacade;
use crate::types::{PoiseData, Result};
use poise::serenity_prelude::{Context, Guild};
use tracing::{info, warn};

/// Botがギルドに参加した、またはBotが起動してギルド情報を受信した時に呼ばれる
pub async fn on_guild_create(
    ctx: &Context,
    guild: &Guild,
    is_new: bool,
    data: &PoiseData,
) -> Result<()> {
    info!(
        guild_id = %guild.id,
        guild_name = %guild.name,
        "ギルド情報を受信しました"
    );

    let app_state = &data.app_state;
    let facade = GuildManagementFacade::new(std::sync::Arc::new(app_state.clone()));
    facade
        .register_new_guild(guild.id.get() as i64, &guild.name)
        .await?;

    info!(
        guild_id = %guild.id,
        guild_name = %guild.name,
        "ギルドを登録または更新しました"
    );

    // Bot新規参加時のみ初期設定メッセージを送信する
    if !is_new {
        info!(
            guild_id = %guild.id,
            "既存ギルドの再受信のため初期設定メッセージ送信をスキップしました"
        );
        return Ok(());
    }

    let Some(system_channel_id) = guild.system_channel_id else {
        warn!(
            guild_id = %guild.id,
            "system_channel_id が未設定のため初期設定メッセージを送信できません"
        );
        return Ok(());
    };

    let init_message = build_init_guide_message(&data.app_state, guild.id.get() as i64).await;

    if let Err(e) = system_channel_id.say(&ctx.http, init_message).await {
        warn!(
            error = %e,
            guild_id = %guild.id,
            channel_id = %system_channel_id,
            "初期設定メッセージ送信に失敗しました"
        );
        return Ok(());
    }

    info!(
        guild_id = %guild.id,
        channel_id = %system_channel_id,
        "初期設定メッセージを送信しました"
    );
    Ok(())
}
