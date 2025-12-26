use crate::models::entities::master::channel_types::{self, Entity as ChannelTypeEntity};
use crate::repository::database::db_compat::Database;
use sea_orm::{DbErr, EntityTrait};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelType {
    pub id: i32,
    pub name: String,
    pub memo: Option<String>,
}

impl From<channel_types::Model> for ChannelType {
    fn from(model: channel_types::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            memo: model.memo,
        }
    }
}

impl Database {
    pub async fn get_channel_types(&self) -> Result<Vec<ChannelType>, DbErr> {
        let models = ChannelTypeEntity::find().all(&self.conn).await?;
        Ok(models.into_iter().map(ChannelType::from).collect())
    }

    pub async fn get_channel_type_by_id(
        &self,
        channel_type: i32,
    ) -> Result<Option<ChannelType>, DbErr> {
        let model = ChannelTypeEntity::find_by_id(channel_type)
            .one(&self.conn)
            .await?;
        Ok(model.map(ChannelType::from))
    }
}
