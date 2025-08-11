use poise::serenity_prelude::all::{Context, Member};

/// Checks if a member has the specified role name
pub async fn has_role(ctx: &Context, member: &Member, role_name: &str) -> bool {
    member.roles.iter().any(|role| {
        if let Some(role) = role.to_role_cached(&ctx.cache) {
            role.name == role_name
        } else {
            false
        }
    })
}

/// Checks if a member has the gbf_bot_control role
pub async fn has_bot_control_permission(ctx: &Context, member: &Member) -> bool {
    has_role(ctx, member, "gbf_bot_control").await
}