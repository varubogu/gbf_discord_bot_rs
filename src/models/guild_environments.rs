use crate::models::entities::guild_master::guild_environments;
use serde::{Deserialize, Serialize};

/// ギルド環境変数のDomain Model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildEnvironments {
    pub guild_id: i64,
    pub key: String,
    pub value: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<guild_environments::Model> for GuildEnvironments {
    fn from(model: guild_environments::Model) -> Self {
        Self {
            guild_id: model.guild_id,
            key: model.key,
            value: model.value,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
