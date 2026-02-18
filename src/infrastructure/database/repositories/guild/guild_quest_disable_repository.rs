use crate::models::entities::guild_master::guild_quest_disables;
use crate::models::entities::guild_master::guild_quest_disables::Entity as GuildQuestDisableEntity;
use crate::repository::GuildQuestDisableRepository;
use crate::types::Result;
use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

#[derive(Default, Debug, Clone, Copy)]
pub struct SeaOrmGuildQuestDisableRepository;

impl SeaOrmGuildQuestDisableRepository {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl GuildQuestDisableRepository for SeaOrmGuildQuestDisableRepository {
    async fn disable_quest<'c, C>(&self, db: &'c C, guild_id: i64, quest_id: i32) -> Result<()>
    where
        C: sea_orm::ConnectionTrait,
    {
        let now = chrono::Utc::now();

        let model = guild_quest_disables::ActiveModel {
            guild_id: Set(guild_id),
            quest_id: Set(quest_id),
            created_at: Set(now),
            updated_at: Set(now),
        };

        model.insert(db).await?;

        Ok(())
    }

    async fn enable_quest<'c, C>(&self, db: &'c C, guild_id: i64, quest_id: i32) -> Result<()>
    where
        C: sea_orm::ConnectionTrait,
    {
        GuildQuestDisableEntity::delete_many()
            .filter(guild_quest_disables::Column::GuildId.eq(guild_id))
            .filter(guild_quest_disables::Column::QuestId.eq(quest_id))
            .exec(db)
            .await?;

        Ok(())
    }

    async fn get_disabled_quest_ids<'c, C>(&self, db: &'c C, guild_id: i64) -> Result<Vec<i32>>
    where
        C: sea_orm::ConnectionTrait,
    {
        let models = GuildQuestDisableEntity::find()
            .filter(guild_quest_disables::Column::GuildId.eq(guild_id))
            .all(db)
            .await?;

        Ok(models.into_iter().map(|m| m.quest_id).collect())
    }

    async fn is_disabled<'c, C>(&self, db: &'c C, guild_id: i64, quest_id: i32) -> Result<bool>
    where
        C: sea_orm::ConnectionTrait,
    {
        let result = GuildQuestDisableEntity::find()
            .filter(guild_quest_disables::Column::GuildId.eq(guild_id))
            .filter(guild_quest_disables::Column::QuestId.eq(quest_id))
            .one(db)
            .await?;

        Ok(result.is_some())
    }
}
