use crate::models::entities::{elements, elements::Entity as ElementEntity};
use crate::repository::database::db_compat::Database;
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Element {
    pub id: i32,
    pub reaction_stamp: Option<String>,
    pub name_jp: String,
    pub name_en: Option<String>,
}

impl From<elements::Model> for Element {
    fn from(model: elements::Model) -> Self {
        Self {
            id: model.id,
            reaction_stamp: model.reaction_stamp,
            name_jp: model.name_jp,
            name_en: model.name_en,
        }
    }
}

impl Database {
    pub async fn get_elements(&self) -> Result<Vec<Element>, DbErr> {
        let models = ElementEntity::find().all(&self.conn).await?;
        Ok(models.into_iter().map(Element::from).collect())
    }

    pub async fn get_element_by_id(&self, id: i32) -> Result<Option<Element>, DbErr> {
        let model = ElementEntity::find_by_id(id).one(&self.conn).await?;
        Ok(model.map(Element::from))
    }

    pub async fn get_element_by_reaction_stamp(
        &self,
        reaction_stamp: &str,
    ) -> Result<Option<Element>, DbErr> {
        let model = ElementEntity::find()
            .filter(elements::Column::ReactionStamp.eq(reaction_stamp))
            .one(&self.conn)
            .await?;
        Ok(model.map(Element::from))
    }
}
