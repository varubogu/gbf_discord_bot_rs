use crate::constants::ROLL_GBF_BOT_CONTROLS;
use crate::types::PoiseContext;
use poise::serenity_prelude::all::Member;
use std::env;

/// Checks if a member has the specified role name
pub async fn has_role(
    ctx: &PoiseContext<'_>,
    member: &Member,
    role_name: &str,
) -> Result<(), String> {
    let guild = ctx.guild().ok_or_else(|| {
        "ギルド情報を取得できませんでした。このコマンドはサーバー内でのみ実行可能です".to_string()
    })?;
    let role = guild
        .role_by_name(role_name)
        .ok_or_else(|| format!("role is not found: '{role_name}'"))?;

    let has_permission = member.roles.iter().any(|role_id| role_id.eq(&role.id));
    if has_permission {
        Ok(())
    } else {
        Err(format!(
            "'{}' is roll '{}' not found.",
            member.display_name(),
            role_name
        )
        .to_string())
    }
}

/// Checks if a member has the gbf_bot_control role
pub async fn has_bot_control_permission(
    ctx: &PoiseContext<'_>,
    member: &Member,
) -> Result<(), String> {
    has_role(ctx, member, ROLL_GBF_BOT_CONTROLS).await
}

/// Checks if the current guild is the bot administrator server
pub async fn is_bot_admin_server(ctx: &PoiseContext<'_>) -> Result<bool, String> {
    let guild_id = ctx.guild_id().ok_or("Guild ID not found")?.to_string();

    // 環境変数から管理者専用サーバーのIDを取得
    let admin_server_id = env::var("BOT_ADMIN_SERVER_ID").unwrap_or_else(|_| String::new());

    if admin_server_id.is_empty() {
        return Ok(false);
    }

    Ok(guild_id == admin_server_id)
}

/// Poise check function: Bot管理者専用サーバーでのみ実行可能
pub async fn check_bot_admin_server(
    ctx: poise::Context<'_, crate::types::PoiseData, crate::types::AppError>,
) -> Result<bool, crate::types::AppError> {
    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| crate::types::AppError::Config {
            message: "このコマンドはサーバー内でのみ実行可能です".to_string(),
        })?
        .to_string();

    let admin_server_id = env::var("BOT_ADMIN_SERVER_ID").unwrap_or_else(|_| String::new());

    if admin_server_id.is_empty() {
        return Err(crate::types::AppError::Config {
            message: "BOT_ADMIN_SERVER_ID が設定されていません".to_string(),
        });
    }

    if guild_id != admin_server_id {
        return Err(crate::types::AppError::Config {
            message: "❌ このコマンドはBot管理者専用サーバーでのみ実行可能です".to_string(),
        });
    }

    Ok(true)
}

/// Poise check function: gbf_bot_control ロール保持者のみ実行可能
pub async fn check_bot_control_role(
    ctx: poise::Context<'_, crate::types::PoiseData, crate::types::AppError>,
) -> Result<bool, crate::types::AppError> {
    let member = ctx
        .author_member()
        .await
        .ok_or_else(|| crate::types::AppError::Config {
            message: "メンバー情報を取得できませんでした".to_string(),
        })?;

    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| crate::types::AppError::Config {
            message: "ギルド情報を取得できませんでした".to_string(),
        })?;

    // Send制約を満たすため、guild()を使わずにHTTP経由でロール情報を取得
    let roles = ctx
        .http()
        .get_guild_roles(guild_id)
        .await
        .map_err(crate::types::AppError::Discord)?;

    let control_role = roles
        .iter()
        .find(|r| r.name == ROLL_GBF_BOT_CONTROLS)
        .ok_or_else(|| crate::types::AppError::Config {
            message: format!("ロール '{ROLL_GBF_BOT_CONTROLS}' が見つかりません"),
        })?;

    let has_permission = member
        .roles
        .iter()
        .any(|role_id| role_id.eq(&control_role.id));

    if !has_permission {
        return Err(crate::types::AppError::Config {
            message: "❌ このコマンドは gbf_bot_control ロール保持者のみ実行可能です".to_string(),
        });
    }

    Ok(true)
}
