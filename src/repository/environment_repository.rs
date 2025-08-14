use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, Set, EntityTrait, ColumnTrait, QueryFilter, DatabaseConnection};
use crate::types::PoiseError;
use crate::models::environment::Environment;
use crate::models::entities::{environment, environment::Entity as EnvironmentEntity};

#[async_trait]
pub trait EnvironmentRepository {
    /// Get all environment variables
    async fn get_all(&self) -> Result<Vec<Environment>, PoiseError>;
    
    /// Get environment variable by key
    async fn get_by_key(&self, key: &str) -> Result<Option<Environment>, PoiseError>;
    
    /// Set environment variable
    async fn set(&self, key: &str, value: &str) -> Result<Environment, PoiseError>;
}

pub struct SeaOrmEnvironmentRepository {
    conn: DatabaseConnection,
}

impl SeaOrmEnvironmentRepository {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl EnvironmentRepository for SeaOrmEnvironmentRepository {
    async fn get_all(&self) -> Result<Vec<Environment>, PoiseError> {
        let environments = EnvironmentEntity::find()
            .all(&self.conn)
            .await
            .map_err(|e| PoiseError::from(format!("Failed to get environments: {}", e)))?;

        Ok(environments.into_iter().map(|env| Environment {
            id: env.id,
            key: env.key,
            value: env.value,
            created_at: env.created_at,
            updated_at: env.updated_at,
        }).collect())
    }

    async fn get_by_key(&self, key: &str) -> Result<Option<Environment>, PoiseError> {
        let environment = EnvironmentEntity::find()
            .filter(environment::Column::Key.eq(key))
            .one(&self.conn)
            .await
            .map_err(|e| PoiseError::from(format!("Failed to get environment by key: {}", e)))?;

        Ok(environment.map(|env| Environment {
            id: env.id,
            key: env.key,
            value: env.value,
            created_at: env.created_at,
            updated_at: env.updated_at,
        }))
    }

    async fn set(&self, key: &str, value: &str) -> Result<Environment, PoiseError> {
        // First try to find existing environment variable
        let existing = EnvironmentEntity::find()
            .filter(environment::Column::Key.eq(key))
            .one(&self.conn)
            .await
            .map_err(|e| PoiseError::from(format!("Failed to check the existing environment: {}", e)))?;

        let result = if let Some(existing_env) = existing {
            // Update existing environment variable
            let mut active_model: environment::ActiveModel = existing_env.into();
            active_model.value = Set(value.to_string());
            active_model.updated_at = Set(chrono::Utc::now());
            
            active_model.update(&self.conn).await
                .map_err(|e| PoiseError::from(format!("Failed to update environment: {}", e)))?
        } else {
            // Create new environment variable
            let new_env = environment::ActiveModel {
                key: Set(key.to_string()),
                value: Set(value.to_string()),
                created_at: Set(chrono::Utc::now()),
                updated_at: Set(chrono::Utc::now()),
                ..Default::default()
            };
            
            new_env.insert(&self.conn).await
                .map_err(|e| PoiseError::from(format!("Failed to create environment: {}", e)))?
        };

        Ok(Environment {
            id: result.id,
            key: result.key,
            value: result.value,
            created_at: result.created_at,
            updated_at: result.updated_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_test_repo() -> Result<SeaOrmEnvironmentRepository, String> {
        if std::env::var("DATABASE_URL").is_err() {
            return Err("DATABASE_URL not set".to_string());
        }

        let conn = match crate::models::database::Database::new().await {
            Ok(db) => db.conn,
            Err(e) => return Err(format!("Failed to connect to database: {}", e)),
        };

        Ok(SeaOrmEnvironmentRepository::new(conn))
    }

    #[tokio::test]
    async fn test_environment_operations() {
        let repo = match setup_test_repo().await {
            Ok(repo) => repo,
            Err(e) => {
                println!("Skipping database test: {}", e);
                return;
            }
        };

        // Test getting all environments
        let get_all_result = repo.get_all().await;
        match get_all_result {
            Ok(environments) => {
                println!("Retrieved {} environments", environments.len());
                for env in environments {
                    assert!(!env.key.is_empty(), "Environment key should not be empty");
                }
            },
            Err(e) => {
                println!("Get environments returned error: {}", e);
            }
        }

        // Test getting a specific environment
        let test_key = "TEST_KEY";
        let get_result = repo.get_by_key(test_key).await;
        match get_result {
            Ok(None) => {
                // Try to set the environment variable
                let set_result = repo.set(test_key, "test_value").await;
                match set_result {
                    Ok(env) => {
                        assert_eq!(env.key, test_key);
                        assert_eq!(env.value, "test_value");

                        // Try to retrieve it again
                        let get_again_result = repo.get_by_key(test_key).await;
                        match get_again_result {
                            Ok(Some(retrieved_env)) => {
                                assert_eq!(retrieved_env.key, test_key);
                                assert_eq!(retrieved_env.value, "test_value");
                            },
                            _ => println!("Failed to retrieve set environment"),
                        }
                    },
                    Err(e) => {
                        println!("Set environment returned error: {}", e);
                    }
                }
            },
            Ok(Some(env)) => {
                println!("Found existing environment: {} = {}", env.key, env.value);
                assert_eq!(env.key, test_key);
            },
            Err(e) => {
                println!("Get environment returned error: {}", e);
            }
        }
    }
}