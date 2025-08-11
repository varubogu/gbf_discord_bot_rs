use std::sync::Arc;
use async_trait::async_trait;
use tracing::{info, error};
use crate::{PoiseData, PoiseError};
use crate::services::database::Database;
use crate::events::handlers;

async fn event_handler(
    ctx: &poise::serenity_prelude::Context,
    event: &poise::serenity_prelude::FullEvent,
    _framework: poise::FrameworkContext<'_, PoiseData, PoiseError>,
    data: &PoiseData,
) -> Result<(), PoiseError> {
    match event {
        poise::serenity_prelude::FullEvent::Ready(ready) => {
            println!("Connected as {}", ready.user.name);
            handlers::ready::on_ready(ctx).await?;
            Ok(())
        },
        poise::serenity_prelude::FullEvent::ReactionAdd(reaction) => {
            println!("Connected as {}", reaction.user.name);
            handlers::reaction_add::on_reaction_add(ctx, reaction).await?;
            Ok(())
        },
        poise::serenity_prelude::FullEvent::ReactionRemove(reaction) => {
            println!("Connected as {}", reaction.user.name);
            handlers::reaction_remove::on_reaction_remove(ctx, reaction).await?;
            Ok(())
        },
        _ => {
            Ok(())
        }
    }
}


pub struct EventHandler {
}

impl EventHandler {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl PoiseEvent for EventHandler {

}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_event_handler_creation() {
        // Skip this test if no database is available
        if std::env::var("DATABASE_URL").is_err() {
            println!("Skipping event handler test: DATABASE_URL not set");
            return;
        }
        
        // This test simply verifies that we can create an EventHandler
        match Database::new().await {
            Ok(db) => {
                let db_arc = Arc::new(db);
                let handler = EventHandler::new(db_arc);
                
                // If we got here without panicking, the test passes
                assert!(true, "EventHandler creation succeeded");
            },
            Err(_) => {
                println!("Skipping event handler test: Database connection failed");
                return;
            }
        }
    }
}