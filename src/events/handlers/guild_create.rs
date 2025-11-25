use crate::repository::database::guild_repository::GuildRepository;
use crate::types::{PoiseData, Result};
use poise::serenity_prelude::{Context, Guild};
use sea_orm::TransactionTrait;
use tracing::{error, info};

/// Botがギルドに参加した、またはBotが起動してギルド情報を受信した時に呼ばれる
pub async fn on_guild_create(ctx: &Context, guild: &Guild, data: &PoiseData) -> Result<()> {
    info!(
        guild_id = %guild.id,
        guild_name = %guild.name,
        "ギルド情報を受信しました"
    );

    let app_state = &data.app_state;
    let txn = app_state.db().begin().await?;

    let result = async {
        let guild_repo = GuildRepository::new(app_state.db().clone());

        // ギルドを自動登録または更新
        guild_repo
            .upsert_with_txn(&txn, guild.id.get() as i64, guild.name.clone())
            .await?;

        Ok::<(), crate::types::AppError>(())
    }
    .await;

    match result {
        Ok(_) => {
            txn.commit().await?;
            info!(
                guild_id = %guild.id,
                guild_name = %guild.name,
                "ギルドを登録または更新しました"
            );
            Ok(())
        }
        Err(e) => {
            error!(
                error = %e,
                guild_id = %guild.id,
                "ギルドの登録または更新に失敗しました"
            );
            txn.rollback().await?;
            Err(e)
        }
    }
}
