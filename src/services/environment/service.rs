use std::sync::Arc;

use crate::services::database::Database;
use super::init::load_from_database_and_update_env;

/// Service function to load environment variables from database
pub async fn load_environment_from_database(db: Arc<Database>) -> Result<(), Box<dyn std::error::Error>> {
    load_from_database_and_update_env(db).await
}