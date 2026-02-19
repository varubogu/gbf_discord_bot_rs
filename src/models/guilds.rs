use crate::infrastructure::database::session::DatabaseSession as Database;
use crate::models::entities::guild_master::guilds::{self, Entity as GuildEntity};
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guild {
    pub guild_id: i64,
    pub name: String,
    pub recruit_channel_id: Option<i64>,
    pub timezone: Option<String>,
    pub default_recruit_duration: Option<i32>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<guilds::Model> for Guild {
    fn from(model: guilds::Model) -> Self {
        Self {
            guild_id: model.guild_id,
            name: model.name,
            recruit_channel_id: model.recruit_channel_id,
            timezone: model.timezone,
            default_recruit_duration: model.default_recruit_duration,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

impl Database {
    pub async fn get_guilds(&self) -> Result<Vec<Guild>, DbErr> {
        let models = GuildEntity::find().all(&self.conn).await?;
        Ok(models.into_iter().map(|model| model.into()).collect())
    }

    pub async fn get_guild_by_id(&self, guild_id: i64) -> Result<Option<Guild>, DbErr> {
        let guild = GuildEntity::find()
            .filter(guilds::Column::GuildId.eq(guild_id))
            .one(&self.conn)
            .await?;

        Ok(guild.map(|g| g.into()))
    }

    pub async fn get_guild_by_name(&self, name: &str) -> Result<Option<Guild>, DbErr> {
        let guild = GuildEntity::find()
            .filter(guilds::Column::Name.eq(name))
            .one(&self.conn)
            .await?;

        Ok(guild.map(|g| g.into()))
    }

    pub async fn get_guilds_with_recruit_channel(&self) -> Result<Vec<Guild>, DbErr> {
        let models = GuildEntity::find()
            .filter(guilds::Column::RecruitChannelId.is_not_null())
            .all(&self.conn)
            .await?;

        Ok(models.into_iter().map(|model| model.into()).collect())
    }
}
