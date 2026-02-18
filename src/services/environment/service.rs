use std::sync::Arc;

use super::init::load_from_database_and_update_env;
use crate::infrastructure::database::repositories::models_database::Database;

/// Service function to load environment variables from database
pub async fn load_environment_from_database(
    db: Arc<Database>,
) -> Result<(), Box<dyn std::error::Error>> {
    load_from_database_and_update_env(db).await
}
