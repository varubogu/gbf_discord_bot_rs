use poise::serenity_prelude::all::{Context, Member};
use crate::types::{PoiseContext, PoiseError};
use crate::utils::constants::ROLL_GBF_BOT_CONTROLS;

/// Checks if a member has the specified role name
pub async fn has_role(ctx: &PoiseContext<'_>, member: &Member, role_name: &str) -> Result<(), String> {
    let guild = ctx.guild().unwrap();
    let role = guild.role_by_name(role_name);
    if role.is_none() {
        return Err(format!("role is not found: '{}'", role_name).to_string())
    }
    let role = role.unwrap();

    let has_permission = member.roles.iter().any(|role_id| {
        role_id.eq(&role.id)
    });
    if has_permission {
        Ok(())
    } else {
        Err(format!("'{}' is roll '{}' not found.", member.display_name(),  role_name).to_string())
    }

}

/// Checks if a member has the gbf_bot_control role
pub async fn has_bot_control_permission(ctx: &PoiseContext<'_>, member: &Member) -> Result<(), String> {
    has_role(ctx, member, ROLL_GBF_BOT_CONTROLS).await
}