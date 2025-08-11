use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use regex::Regex;
use tracing::{info, error};

use crate::models::Database;

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
        let dotenv_path = Path::new(&config_folder).join(".env");
        
        // Load .env file
        match dotenv::from_path(&dotenv_path) {
            Ok(_) => {
                info!("Loaded environment from {}", dotenv_path.display());
                
                // Load all environment variables into our HashMap
                for (key, value) in env::vars() {
                    self.set(&key, &value).await;
                }
                
                Ok(())
            },
            Err(e) => {
                error!("Failed to load .env file: {}", e);
                Err(Box::new(EnvironmentError { 
                    message: format!("Failed to load .env file: {}", e) 
                }))
            }
        }
    }
    
    pub async fn load_from_database(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(db) = &self.db {
            match db.get_environments().await {
                Ok(environments) => {
                    let mut vars = self.variables.write().await;
                    for env in &environments {
                        vars.insert(env.key.clone(), env.value.clone());
                    }
                    info!("Loaded {} environment variables from database", environments.len());
                    Ok(())
                },
                Err(e) => {
                    error!("Failed to load environment variables from database: {}", e);
                    Err(Box::new(EnvironmentError { 
                        message: format!("Database error: {}", e) 
                    }))
                }
            }
        } else {
            error!("No database connection available");
            Err(Box::new(EnvironmentError { 
                message: "No database connection available".to_string() 
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
    
    pub async fn replace_variables(&self, text: &str) -> Result<String, Box<dyn std::error::Error>> {
        let re = Regex::new(r"\$\{(\w+)\}").unwrap();
        let vars = self.variables.read().await;
        
        let mut result = text.to_string();
        let mut missing_keys = Vec::new();
        
        for cap in re.captures_iter(text) {
            let full_match = cap.get(0).unwrap().as_str();
            let key = cap.get(1).unwrap().as_str();
            
            if let Some(value) = vars.get(key) {
                result = result.replace(full_match, value);
            } else {
                missing_keys.push(key.to_string());
            }
        }
        
        if !missing_keys.is_empty() {
            return Err(Box::new(EnvironmentError { 
                message: format!("Missing environment variables: {}", missing_keys.join(", ")) 
            }));
        }
        
        Ok(result)
    }
}

// Create a singleton instance
lazy_static::lazy_static! {
    pub static ref ENV: Environment = Environment::new();
}

// Initialize the environment
pub async fn init_environment(db: Option<Arc<Database>>) -> Result<(), Box<dyn std::error::Error>> {
    // Load from .env file
    ENV.load_from_env_file().await?;
    
    // If database is provided, set it and load from database
    if let Some(db) = db {
        // We can't modify the ENV directly since it's a lazy_static,
        // but we can load the database values
        let env = Environment::with_database(db);
        if let Err(e) = env.load_from_database().await {
            error!("Failed to load environment from database: {}", e);
            // Continue anyway, as we at least have the .env values
        }
        
        // Copy values from the temporary environment to the singleton
        let vars = env.get_all().await;
        for (key, value) in vars.iter() {
            ENV.set(key, value).await;
        }
    }
    
    Ok(())
}