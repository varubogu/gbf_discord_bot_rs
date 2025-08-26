use crate::models::entities::{guilds, guilds::Entity as GuildEntity};
use crate::repository::database::db_compat::Database;
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guild {
    pub id: i32,
    pub discord_guild_id: i64,
    pub guild_name: String,
    pub recruit_channel_id: Option<i64>,
    pub notification_channel_id: Option<i64>,
    pub timezone: Option<String>,
    pub default_recruit_duration: Option<i32>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<guilds::Model> for Guild {
    fn from(model: guilds::Model) -> Self {
        Self {
            id: model.id,
            discord_guild_id: model.discord_guild_id,
            guild_name: model.guild_name,
            recruit_channel_id: model.recruit_channel_id,
            notification_channel_id: model.notification_channel_id,
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

    pub async fn get_guild_by_id(&self, id: i32) -> Result<Option<Guild>, DbErr> {
        let guild = GuildEntity::find()
            .filter(guilds::Column::Id.eq(id))
            .one(&self.conn)
            .await?;

        Ok(guild.map(|g| g.into()))
    }

    pub async fn get_guild_by_discord_id(
        &self,
        discord_guild_id: i64,
    ) -> Result<Option<Guild>, DbErr> {
        let guild = GuildEntity::find()
            .filter(guilds::Column::DiscordGuildId.eq(discord_guild_id))
            .one(&self.conn)
            .await?;

        Ok(guild.map(|g| g.into()))
    }

    pub async fn get_guild_by_name(&self, guild_name: &str) -> Result<Option<Guild>, DbErr> {
        let guild = GuildEntity::find()
            .filter(guilds::Column::GuildName.eq(guild_name))
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

    pub async fn get_guilds_with_notification_channel(&self) -> Result<Vec<Guild>, DbErr> {
        let models = GuildEntity::find()
            .filter(guilds::Column::NotificationChannelId.is_not_null())
            .all(&self.conn)
            .await?;

        Ok(models.into_iter().map(|model| model.into()).collect())
    }
}
