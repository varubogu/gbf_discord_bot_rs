use poise::serenity_prelude::all::{
    Context, GuildId, CreateCommand, CreateCommandOption, CommandOptionType,
};
use tracing::{error, info};
use std::env;

/// Register all slash commands with Discord
pub async fn register_commands(ctx: &Context) -> Result<(), String> {
    let guild_id = GuildId::new(
        env::var("GUILD_ID")
            .expect("Expected GUILD_ID in environment")
            .parse()
            .expect("GUILD_ID must be a valid ID")
    );
    
    let commands = guild_id.set_commands(&ctx.http, vec![
        CreateCommand::new("recruit")
            .description("マルチバトルを募集します")
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "quest",
                    "募集するクエスト"
                )
                .required(true)
            )
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "battle_type",
                    "クエストの攻略方法"
                )
                .add_int_choice("デフォルト", 0)
                .add_int_choice("全属性", 1)
                .add_int_choice("火属性", 2)
                .add_int_choice("水属性", 3)
                .add_int_choice("土属性", 4)
                .add_int_choice("風属性", 5)
                .add_int_choice("光属性", 6)
                .add_int_choice("闇属性", 7)
            )
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "event_date",
                    "クエスト開始日時(月/日 時:分)"
                )
            ),
        CreateCommand::new("environ_load")
            .description("環境変数読み込み"),
        CreateCommand::new("help")
            .description("ヘルプを表示します")
    ]).await;
    
    match commands {
        Ok(_) => {
            info!("Registered slash commands");
            Ok(())
        },
        Err(e) => {
            error!("Error registering slash commands: {:?}", e);
            Err(format!("Failed to register slash commands: {}", e))
        }
    }
}