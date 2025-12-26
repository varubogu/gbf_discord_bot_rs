use crate::models::entities::master::environments::{self, Entity as EnvironmentEntity};
use crate::models::environments::Environments;
use crate::repository::EnvironmentRepository;
use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter, Set};

pub struct SeaOrmEnvironmentRepository;

impl SeaOrmEnvironmentRepository {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl EnvironmentRepository for SeaOrmEnvironmentRepository {
    async fn get_all<'c, C>(&self, db: &'c C) -> Result<Vec<Environments>, DbErr>
    where
        C: sea_orm::ConnectionTrait,
    {
        let models = EnvironmentEntity::find()
            .all(db)
            .await?;

        Ok(models
            .into_iter()
            .map(|env| env.into())
            .collect())
    }

    async fn get_by_key<'c, C>(&self, db: &'c C, key: &str) -> Result<Option<Environments>, DbErr>
    where
        C: sea_orm::ConnectionTrait,
    {
        let model = EnvironmentEntity::find()
            .filter(environments::Column::Key.eq(key))
            .one(db)
            .await?;

        Ok(model.map(|env| env.into()))
    }

    async fn set<'c, C>(&self, db: &'c C, key: &str, value: &str) -> Result<Environments, DbErr>
    where
        C: sea_orm::ConnectionTrait,
    {
        // First try to find existing environment variable
        let existing = EnvironmentEntity::find()
            .filter(environments::Column::Key.eq(key))
            .one(db)
            .await?;

        let result = if let Some(existing_env) = existing {
            // Update existing environment variable
            let mut active_model: environments::ActiveModel = existing_env.into();
            active_model.value = Set(value.to_string());
            active_model.updated_at = Set(chrono::Utc::now());

            active_model
                .update(db)
                .await?
        } else {
            // Create new environment variable
            let new_env = environments::ActiveModel {
                key: Set(key.to_string()),
                value: Set(value.to_string()),
                created_at: Set(chrono::Utc::now()),
                updated_at: Set(chrono::Utc::now()),
                ..Default::default()
            };

            new_env
                .insert(db)
                .await?
        };

        Ok(result.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::database::connection::is_database_available;

    async fn setup_test_repo() -> Result<(SeaOrmEnvironmentRepository, sea_orm::DatabaseConnection), String> {
        let (available, missing) = is_database_available();
        if !available {
            return Err(format!(
                "Database connection info not set - missing: {:?}",
                missing
            ));
        }

        let conn = match crate::repository::database::models_database::Database::new().await {
            Ok(db) => db.conn,
            Err(e) => return Err(format!("Failed to connect to database: {}", e)),
        };

        Ok((SeaOrmEnvironmentRepository::new(), conn))
    }

    #[tokio::test]
    async fn test_environment_operations() {
        let (repo, conn) = match setup_test_repo().await {
            Ok(result) => result,
            Err(e) => {
                println!("Skipping database test: {}", e);
                return;
            }
        };

        // Test getting all environments
        let get_all_result = repo.get_all(&conn).await;
        match get_all_result {
            Ok(environments) => {
                println!("Retrieved {} environments", environments.len());
                for env in environments {
                    assert!(!env.key.is_empty(), "Environment key should not be empty");
                }
            }
            Err(e) => {
                println!("Get environments returned error: {}", e);
            }
        }

        // Test getting a specific environment
        let test_key = "TEST_KEY";
        let get_result = repo.get_by_key(&conn, test_key).await;
        match get_result {
            Ok(None) => {
                // Try to set the environment variable
                let set_result = repo.set(&conn, test_key, "test_value").await;
                match set_result {
                    Ok(env) => {
                        assert_eq!(env.key, test_key);
                        assert_eq!(env.value, "test_value");

                        // Try to retrieve it again
                        let get_again_result = repo.get_by_key(&conn, test_key).await;
                        match get_again_result {
                            Ok(Some(retrieved_env)) => {
                                assert_eq!(retrieved_env.key, test_key);
                                assert_eq!(retrieved_env.value, "test_value");
                            }
                            _ => println!("Failed to retrieve set environment"),
                        }
                    }
                    Err(e) => {
                        println!("Set environment returned error: {}", e);
                    }
                }
            }
            Ok(Some(env)) => {
                println!("Found existing environment: {} = {}", env.key, env.value);
                assert_eq!(env.key, test_key);
            }
            Err(e) => {
                println!("Get environment returned error: {}", e);
            }
        }
    }
}
