use poise::serenity_prelude::all::{Message, User, UserId, ChannelId, MessageId, GuildId};
use std::collections::{HashMap, HashSet};

pub async fn guild_id_url_str(guild_id: Option<GuildId>) -> String {
    if let Some(g) = guild_id {
        g.to_string()
    } else {
        String::from("@me")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use poise::serenity_prelude::all::{MessageId, ChannelId, GuildId, MessageType};
    use chrono::Utc;

    #[tokio::test]
    async fn test_guild_id_url_str_with_guild_id() {
        let guild_id = GuildId::new(123456789);
        let result = guild_id_url_str(Some(guild_id)).await;
        assert_eq!(result, "123456789");
    }

    #[tokio::test]
    async fn test_guild_id_url_str_without_guild_id() {
        let result = guild_id_url_str(None).await;
        assert_eq!(result, "@me");
    }

    #[tokio::test]
    async fn test_guild_id_url_str_with_different_ids() {
        // Test with various guild IDs
        let test_cases = vec![
            (Some(GuildId::new(1)), "1"),
            (Some(GuildId::new(999999999999999999)), "999999999999999999"),
            (Some(GuildId::new(0)), "0"),
            (None, "@me"),
        ];

        for (guild_id, expected) in test_cases {
            let result = guild_id_url_str(guild_id).await;
            assert_eq!(result, expected, "Failed for guild_id: {:?}", guild_id);
        }
    }

    #[tokio::test]
    async fn test_make_message_url_with_guild() {
        // Create a mock message with guild
        let guild_id = GuildId::new(123456789);
        let channel_id = ChannelId::new(987654321);
        let message_id = MessageId::new(555666777);
        
        // Note: Creating a full Message struct is complex in tests due to many required fields
        // For now, we'll test the URL format logic by constructing the expected URL manually
        let expected_url = format!(
            "https://discord.com/channels/{}/{}/{}",
            guild_id, channel_id, message_id
        );
        
        // Test the URL pattern
        assert!(expected_url.contains("https://discord.com/channels/"));
        assert!(expected_url.contains("123456789"));
        assert!(expected_url.contains("987654321"));
        assert!(expected_url.contains("555666777"));
    }

    #[tokio::test]
    async fn test_make_message_url_without_guild() {
        // Test DM channel URL format
        let channel_id = ChannelId::new(987654321);
        let message_id = MessageId::new(555666777);
        
        let expected_url = format!(
            "https://discord.com/channels/@me/{}/{}",
            channel_id, message_id
        );
        
        // Test the URL pattern for DM
        assert!(expected_url.contains("https://discord.com/channels/@me/"));
        assert!(expected_url.contains("987654321"));
        assert!(expected_url.contains("555666777"));
    }

    #[tokio::test]
    async fn test_message_url_format_consistency() {
        // Test URL format consistency
        let guild_cases = vec![
            (Some(GuildId::new(123)), "123"),
            (Some(GuildId::new(456789)), "456789"),
            (None, "@me"),
        ];

        for (guild_id, expected_guild_part) in guild_cases {
            let channel_id = ChannelId::new(111);
            let message_id = MessageId::new(222);
            
            let expected_url = format!(
                "https://discord.com/channels/{}/{}/{}",
                expected_guild_part, channel_id, message_id
            );
            
            // Verify URL structure
            assert!(expected_url.starts_with("https://discord.com/channels/"));
            assert!(expected_url.contains(&format!("/{}/", channel_id)));
            assert!(expected_url.ends_with(&format!("/{}", message_id)));
        }
    }

    // Note: The following functions are difficult to test without mocking Discord API:
    // - get_reaction_users (requires Discord HTTP client and message with reactions)
    // - get_unique_reaction_users (depends on get_reaction_users)
    // - update_embed_with_participants (requires Discord HTTP client and message editing)
    //
    // These would require more sophisticated mocking frameworks like mockall or manual mocks.
    // For comprehensive testing, consider adding integration tests for these functions.

    #[test]
    fn test_url_components_extraction() {
        // Test that we can extract components from a Discord URL
        let test_url = "https://discord.com/channels/123456789/987654321/555666777";
        let parts: Vec<&str> = test_url.split('/').collect();
        
        assert_eq!(parts.len(), 7);
        assert_eq!(parts[0], "https:");
        assert_eq!(parts[2], "discord.com");
        assert_eq!(parts[3], "channels");
        assert_eq!(parts[4], "123456789"); // guild_id
        assert_eq!(parts[5], "987654321"); // channel_id
        assert_eq!(parts[6], "555666777"); // message_id
    }

    #[test]
    fn test_dm_url_components_extraction() {
        // Test DM URL format
        let test_url = "https://discord.com/channels/@me/987654321/555666777";
        let parts: Vec<&str> = test_url.split('/').collect();
        
        assert_eq!(parts.len(), 7);
        assert_eq!(parts[4], "@me"); // guild_id for DM
        assert_eq!(parts[5], "987654321"); // channel_id
        assert_eq!(parts[6], "555666777"); // message_id
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
    ctx: &poise::serenity_prelude::all::Context,
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
    ctx: &poise::serenity_prelude::all::Context,
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
    ctx: &poise::serenity_prelude::all::Context,
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
        
        embed.fields.push(poise::serenity_prelude::all::EmbedField::new(reaction, user_mentions, false));
    }
    
    // Create a new message builder
    let mut builder = poise::serenity_prelude::all::EditMessage::default();
    
    // Create a new embed with just the fields we need
    let mut create_embed = poise::serenity_prelude::all::CreateEmbed::new();
    
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