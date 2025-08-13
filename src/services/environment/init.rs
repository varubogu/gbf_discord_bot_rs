use std::sync::Arc;

use crate::repository::Database;
use super::environment::Environment;

// Create a singleton instance
lazy_static::lazy_static! {
    pub static ref ENV: Environment = Environment::new();
}

// Initialize the environment
pub async fn init_environment(db: Option<Arc<Database>>) -> Result<(), Box<dyn std::error::Error>> {
    // Load from .env file
    ENV.load_from_env_file().await?;
    
    // If database is provided, set it and load from database
    if let Some(db) = db {
        load_from_database_and_update_env(db).await?;
    }
    
    Ok(())
}

// Load environment variables from database and update ENV singleton
pub async fn load_from_database_and_update_env(db: Arc<Database>) -> Result<(), Box<dyn std::error::Error>> {
    // We can't modify the ENV directly since it's a lazy_static,
    // but we can load the database values
    let env = Environment::with_database(db);
    
    // Load from database
    env.load_from_database().await?;
    
    // Copy values from the temporary environment to the singleton
    let vars = env.get_all().await;
    for (key, value) in vars.iter() {
        ENV.set(key, value).await;
    }
    
    Ok(())
}