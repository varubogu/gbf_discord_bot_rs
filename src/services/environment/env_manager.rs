use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

use crate::infrastructure::database::repositories::models_database::Database;

lazy_static! {
    /// 環境変数置換パターン（${VARIABLE_NAME}）
    static ref RE_VARIABLE: Regex = Regex::new(r"\$\{([A-Za-z0-9_\-\.]+)\}")
        .expect("環境変数置換パターンのRegexが無効です");
}

#[derive(Debug)]
pub struct EnvironmentError {
    pub message: String,
}

impl std::fmt::Display for EnvironmentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Environment error: {}", self.message)
    }
}

impl std::error::Error for EnvironmentError {}

pub struct Environment {
    variables: RwLock<HashMap<String, String>>,
    db: Option<Arc<Database>>,
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

impl Environment {
    pub fn new() -> Self {
        Self {
            variables: RwLock::new(HashMap::new()),
            db: None,
        }
    }

    pub fn with_database(db: Arc<Database>) -> Self {
        Self {
            variables: RwLock::new(HashMap::new()),
            db: Some(db),
        }
    }

    pub async fn load_from_env_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Get config folder from environment or use default
        let config_folder = env::var("CONFIG_FOLDER").unwrap_or_else(|_| ".".to_string());
        let dotenv_path = Path::new(&config_folder).join(".env.app");

        // Load .env file
        match dotenv::from_path(&dotenv_path) {
            Ok(_) => {
                info!("Loaded environment from {}", dotenv_path.display());

                // Load all environment variables into our HashMap
                let env_vars: Vec<(String, String)> = env::vars().collect();
                for (key, value) in env_vars {
                    self.set(&key, &value).await;
                }

                Ok(())
            }
            Err(e) => {
                error!("Failed to load .env file: {}", e);
                Err(Box::new(EnvironmentError {
                    message: format!("Failed to load .env file: {e}"),
                }))
            }
        }
    }

    pub async fn load_from_database(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(_db) = &self.db {
            // TODO: Implement database environment loading when environment repository is available
            info!("Database environment loading not yet implemented");
            Ok(())
        } else {
            error!("No database connection available");
            Err(Box::new(EnvironmentError {
                message: "No database connection available".to_string(),
            }))
        }
    }

    pub async fn set(&self, key: &str, value: &str) {
        let mut vars = self.variables.write().await;
        vars.insert(key.to_string(), value.to_string());
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        let vars = self.variables.read().await;
        vars.get(key).cloned()
    }

    pub async fn get_or(&self, key: &str, default: &str) -> String {
        match self.get(key).await {
            Some(value) => value,
            None => default.to_string(),
        }
    }

    pub async fn delete(&self, key: &str) {
        let mut vars = self.variables.write().await;
        vars.remove(key);
    }

    pub async fn clear(&self) {
        let mut vars = self.variables.write().await;
        vars.clear();
    }

    pub async fn get_all(&self) -> HashMap<String, String> {
        let vars = self.variables.read().await;
        vars.clone()
    }

    pub async fn replace_variables(
        &self,
        text: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let vars = self.variables.read().await;

        let mut result = text.to_string();
        let mut missing_keys = Vec::new();

        for cap in RE_VARIABLE.captures_iter(text) {
            if let (Some(full_match), Some(key)) = (cap.get(0), cap.get(1)) {
                let full_match = full_match.as_str();
                let key = key.as_str();

                if let Some(value) = vars.get(key) {
                    result = result.replace(full_match, value);
                } else {
                    missing_keys.push(key.to_string());
                }
            }
        }

        if !missing_keys.is_empty() {
            return Err(Box::new(EnvironmentError {
                message: format!("Missing environment variables: {}", missing_keys.join(", ")),
            }));
        }

        Ok(result)
    }

    /// 個別のデータベース環境変数から PostgreSQL URL を構築
    pub async fn get_database_url(&self) -> Result<String, EnvironmentError> {
        // 個別の環境変数から構築
        self.build_database_url().await
    }

    /// 個別のデータベース環境変数から PostgreSQL URL を構築
    pub async fn build_database_url(&self) -> Result<String, EnvironmentError> {
        let host = self.get("DB_HOST").await.ok_or_else(|| EnvironmentError {
            message: "DB_HOST is required".to_string(),
        })?;

        let user = self.get("DB_USER").await.ok_or_else(|| EnvironmentError {
            message: "DB_USER is required".to_string(),
        })?;

        let password = self
            .get("DB_PASSWORD")
            .await
            .ok_or_else(|| EnvironmentError {
                message: "DB_PASSWORD is required".to_string(),
            })?;

        let database = self.get("DB_NAME").await.ok_or_else(|| EnvironmentError {
            message: "DB_NAME is required".to_string(),
        })?;

        // ポートは省略可能（デフォルト: 5432）
        let port = self
            .get("DB_PORT")
            .await
            .unwrap_or_else(|| "5432".to_string());

        Ok(format!(
            "postgresql://{user}:{password}@{host}:{port}/{database}"
        ))
    }
}
