use crate::facades::guild::guild_management_facade::GuildManagementFacade;
use crate::types::{PoiseData, Result};
use poise::serenity_prelude::{Context, Guild};
use tracing::info;

/// Botがギルドに参加した、またはBotが起動してギルド情報を受信した時に呼ばれる
pub async fn on_guild_create(_ctx: &Context, guild: &Guild, data: &PoiseData) -> Result<()> {
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
    Ok(())
}
