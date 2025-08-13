use serde::{Deserialize, Serialize};
use crate::models::entities::{environment, Environment as EnvironmentEntity};
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, ActiveModelTrait, Set, DbErr, IntoActiveModel, ActiveModelBehavior, TransactionTrait};
use crate::models::database::Database;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub id: i32,
    pub key: String,
    pub value: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<environment::Model> for Environment {
    fn from(model: environment::Model) -> Self {
        Self {
            id: model.id,
            key: model.key,
            value: model.value,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

impl Database {
    pub async fn get_environments(&self) -> Result<Vec<Environment>, DbErr> {
        let models = EnvironmentEntity::find()
            .all(&self.conn)
            .await?;
            
        Ok(models.into_iter().map(|model| model.into()).collect())
    }
    
    pub async fn get_environment(&self, key: &str) -> Result<Option<Environment>, DbErr> {
        let model = EnvironmentEntity::find()
            .filter(environment::Column::Key.eq(key))
            .one(&self.conn)
            .await?;
            
        Ok(model.map(|m| m.into()))
    }
    
    pub async fn set_environment(&self, key: &str, value: &str) -> Result<Environment, DbErr> {
        // Start a transaction
        let txn = self.conn.begin().await?;
        
        // Try to find existing environment with the key
        let existing = EnvironmentEntity::find()
            .filter(environment::Column::Key.eq(key))
            .one(&txn)
            .await?;
            
        let result = if let Some(existing) = existing {
            // Update existing
            let mut active_model: environment::ActiveModel = existing.into_active_model();
            active_model.value = Set(value.to_string());
            active_model.update(&txn).await?
        } else {
            // Create new
            let active_model = environment::ActiveModel {
                key: Set(key.to_string()),
                value: Set(value.to_string()),
                ..Default::default()
            };
            active_model.insert(&txn).await?
        };
        
        // Commit the transaction
        txn.commit().await?;
        
        Ok(result.into())
    }
}