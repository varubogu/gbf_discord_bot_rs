use serenity::all::{Message, User, UserId, ChannelId, MessageId, GuildId};
use std::collections::{HashMap, HashSet};

pub async fn guild_id_url_str(guild_id: Option<GuildId>) -> String {
    if let Some(g) = guild_id {
        g.to_string()
    } else {
        String::from("@me")
    }
}

/// Creates a URL to a Discord message
pub async fn make_message_url(message: &Message) -> String {
    format!(
        "https://discord.com/channels/{}/{}/{}",
        guild_id_url_str(message.guild_id).await,
        message.channel_id,
        message.id
    )
}

/// Gets all users who reacted to a message, grouped by reaction
pub async fn get_reaction_users(
    ctx: &serenity::all::Context,
    message: &Message,
) -> Result<HashMap<String, Vec<User>>, String> {
    let mut result = HashMap::new();
    
    for reaction in &message.reactions {
        let emoji_str = reaction.reaction_type.to_string();
        let mut users = Vec::new();
        
        // Get users who reacted with this emoji
        let reaction_users = match message.reaction_users(
            &ctx.http,
            reaction.reaction_type.clone(),
            None,
            None,
        ).await {
            Ok(users) => users,
            Err(e) => {
                tracing::error!("Error fetching reaction users: {:?}", e);
                continue;
            }
        };
        
        // Filter out the bot
        for user in reaction_users {
            if user.id != ctx.cache.current_user().id {
                users.push(user);
            }
        }
        
        result.insert(emoji_str, users);
    }
    
    Ok(result)
}

/// Gets all unique users who reacted to a message
pub async fn get_unique_reaction_users(
    ctx: &serenity::all::Context,
    message: &Message,
) -> Result<HashSet<UserId>, String> {
    let reactions = get_reaction_users(ctx, message).await?;
    
    let mut unique_users = HashSet::new();
    for (_, users) in reactions {
        for user in users {
            unique_users.insert(user.id);
        }
    }
    
    Ok(unique_users)
}

/// Updates an embed with reaction participants
pub async fn update_embed_with_participants(
    ctx: &serenity::all::Context,
    message: &Message,
    reactions: HashMap<String, Vec<User>>,
) -> Result<(), String> {
    // Get the first embed
    if message.embeds.is_empty() {
        return Ok(());
    }
    
    let mut embed = message.embeds[0].clone();
    
    // Clear existing fields
    embed.fields.clear();
    
    // Add fields for each reaction
    for (reaction, users) in &reactions {
        let mut user_mentions = String::new();
        
        for user in users {
            if !user_mentions.is_empty() {
                user_mentions.push_str("  ");
            }
            user_mentions.push_str(&format!("<@{}>", user.id));
        }
        
        if user_mentions.is_empty() {
            user_mentions = "無し".to_string();
        }
        
        embed.fields.push(serenity::all::EmbedField::new(reaction, user_mentions, false));
    }
    
    // Create a new message builder
    let mut builder = serenity::all::EditMessage::default();
    
    // Create a new embed with just the fields we need
    let mut create_embed = serenity::all::CreateEmbed::new();
    
    // Copy title and description if they exist
    if let Some(title) = &embed.title {
        create_embed = create_embed.title(title);
    }
    if let Some(description) = &embed.description {
        create_embed = create_embed.description(description);
    }
    if let Some(colour) = embed.colour {
        create_embed = create_embed.color(colour);
    }
    
    // Add all the fields
    for field in &embed.fields {
        create_embed = create_embed.field(&field.name, &field.value, field.inline);
    }
    
    builder = builder.embed(create_embed);
    
    if let Err(e) = message.channel_id.edit_message(&ctx.http, message.id, builder).await {
        tracing::error!("Error updating message embed: {:?}", e);
        return Err(format!("Failed to update a message: {}", e));
    }
    
    Ok(())
}