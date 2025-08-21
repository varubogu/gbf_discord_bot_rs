use std::env;
use tracing::{error, info};

#[derive(Debug)]
pub struct DatabaseConnectionError {
    pub message: String,
}

impl std::fmt::Display for DatabaseConnectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Database connection error: {}", self.message)
    }
}

impl std::error::Error for DatabaseConnectionError {}

/// 環境変数からPostgreSQLのデータベースURLを構築する
/// ポートは省略可能で、デフォルトは5432
pub fn build_database_url() -> Result<String, DatabaseConnectionError> {
    // 環境変数をチェック
    let (available, missing) = is_database_available();
    if !available {
        return Err(DatabaseConnectionError {
            message: format!(
                "Missing required database environment variables: {}",
                missing.join(", ")
            ),
        });
    }

    let host = env::var("DB_HOST").map_err(|_| DatabaseConnectionError {
        message: "DB_HOST is required".to_string(),
    })?;

    let user = env::var("DB_USER").map_err(|_| DatabaseConnectionError {
        message: "DB_USER is required".to_string(),
    })?;

    let password = env::var("DB_PASSWORD").map_err(|_| DatabaseConnectionError {
        message: "DB_PASSWORD is required".to_string(),
    })?;

    let database = env::var("DB_NAME").map_err(|_| DatabaseConnectionError {
        message: "DB_NAME is required".to_string(),
    })?;

    // ポートは省略可能（デフォルト: 5432）
    let port = env::var("DB_PORT").unwrap_or_else(|_| "5432".to_string());

    let url = format!(
        "postgresql://{}:{}@{}:{}/{}",
        user, password, host, port, database
    );
    info!(
        "Built database URL for host: {} port: {} database: {}",
        host, port, database
    );

    Ok(url)
}

/// 環境変数でのデータベース接続情報の利用可能性をチェック（テスト用）
/// 戻り値: (利用可能かどうか, 不足している環境変数のリスト)
pub fn is_database_available() -> (bool, Vec<String>) {
    let mut missing_vars = Vec::new();

    if env::var("DB_HOST").is_err() {
        missing_vars.push("DB_HOST".to_string());
    }
    if env::var("DB_USER").is_err() {
        missing_vars.push("DB_USER".to_string());
    }
    if env::var("DB_PASSWORD").is_err() {
        missing_vars.push("DB_PASSWORD".to_string());
    }
    if env::var("DB_NAME").is_err() {
        missing_vars.push("DB_NAME".to_string());
    }

    (missing_vars.is_empty(), missing_vars)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    /// テスト用にクリーンな環境を作る
    fn setup_clean_env() {
        unsafe {
            env::remove_var("DB_HOST");
            env::remove_var("DB_USER");
            env::remove_var("DB_PASSWORD");
            env::remove_var("DB_NAME");
            env::remove_var("DB_PORT");
        }
    }

    /// テスト用に全ての必須環境変数を設定
    fn setup_complete_env() {
        unsafe {
            env::set_var("DB_HOST", "localhost");
            env::set_var("DB_USER", "test_user");
            env::set_var("DB_PASSWORD", "test_password");
            env::set_var("DB_NAME", "test_db");
            env::set_var("DB_PORT", "5432");
        }
    }

    #[test]
    fn test_build_database_url() {
        // このテストは実際の環境変数に依存するため、
        // 環境変数が設定されていない場合はスキップ
        let (available, missing) = is_database_available();
        if !available {
            println!(
                "Skipping database URL build test: missing variables: {:?}",
                missing
            );
            return;
        }

        let result = build_database_url();
        assert!(result.is_ok(), "Database URL should be built successfully");

        let url = result.unwrap();
        assert!(
            url.starts_with("postgresql://"),
            "URL should start with postgresql://"
        );
        assert!(url.contains("@"), "URL should contain '@' separator");
        assert!(url.contains(":"), "URL should contain ':' separator");
    }

    #[test]
    fn test_is_database_available_all_present() {
        setup_complete_env();

        let (available, missing) = is_database_available();

        assert!(
            available,
            "All database environment variables should be available"
        );
        assert!(
            missing.is_empty(),
            "No environment variables should be missing"
        );

        setup_clean_env();
    }

    #[test]
    fn test_is_database_available_all_missing() {
        setup_clean_env();

        let (available, missing) = is_database_available();

        assert!(
            !available,
            "Database should not be available when all variables are missing"
        );
        assert_eq!(missing.len(), 4, "Should report 4 missing variables");
        assert!(
            missing.contains(&"DB_HOST".to_string()),
            "DB_HOST should be in missing list"
        );
        assert!(
            missing.contains(&"DB_USER".to_string()),
            "DB_USER should be in missing list"
        );
        assert!(
            missing.contains(&"DB_PASSWORD".to_string()),
            "DB_PASSWORD should be in missing list"
        );
        assert!(
            missing.contains(&"DB_NAME".to_string()),
            "DB_NAME should be in missing list"
        );
    }

    #[test]
    fn test_is_database_available_single_missing_db_host() {
        setup_complete_env();
        unsafe {
            env::remove_var("DB_HOST");
        }

        let (available, missing) = is_database_available();

        assert!(
            !available,
            "Database should not be available when DB_HOST is missing"
        );
        assert_eq!(missing.len(), 1, "Should report 1 missing variable");
        assert!(
            missing.contains(&"DB_HOST".to_string()),
            "DB_HOST should be in missing list"
        );

        setup_clean_env();
    }

    #[test]
    fn test_is_database_available_single_missing_db_user() {
        setup_complete_env();
        unsafe {
            env::remove_var("DB_USER");
        }

        let (available, missing) = is_database_available();

        assert!(
            !available,
            "Database should not be available when DB_USER is missing"
        );
        assert_eq!(missing.len(), 1, "Should report 1 missing variable");
        assert!(
            missing.contains(&"DB_USER".to_string()),
            "DB_USER should be in missing list"
        );

        setup_clean_env();
    }

    #[test]
    fn test_is_database_available_single_missing_db_password() {
        setup_complete_env();
        unsafe {
            env::remove_var("DB_PASSWORD");
        }

        let (available, missing) = is_database_available();

        assert!(
            !available,
            "Database should not be available when DB_PASSWORD is missing"
        );
        assert_eq!(missing.len(), 1, "Should report 1 missing variable");
        assert!(
            missing.contains(&"DB_PASSWORD".to_string()),
            "DB_PASSWORD should be in missing list"
        );

        setup_clean_env();
    }

    #[test]
    fn test_is_database_available_single_missing_db_name() {
        setup_complete_env();
        unsafe {
            env::remove_var("DB_NAME");
        }

        let (available, missing) = is_database_available();

        assert!(
            !available,
            "Database should not be available when DB_NAME is missing"
        );
        assert_eq!(missing.len(), 1, "Should report 1 missing variable");
        assert!(
            missing.contains(&"DB_NAME".to_string()),
            "DB_NAME should be in missing list"
        );

        setup_clean_env();
    }

    #[test]
    fn test_is_database_available_multiple_missing_host_user() {
        setup_complete_env();
        unsafe {
            env::remove_var("DB_HOST");
            env::remove_var("DB_USER");
        }

        let (available, missing) = is_database_available();

        assert!(
            !available,
            "Database should not be available when DB_HOST and DB_USER are missing"
        );
        assert_eq!(missing.len(), 2, "Should report 2 missing variables");
        assert!(
            missing.contains(&"DB_HOST".to_string()),
            "DB_HOST should be in missing list"
        );
        assert!(
            missing.contains(&"DB_USER".to_string()),
            "DB_USER should be in missing list"
        );

        setup_clean_env();
    }

    #[test]
    fn test_is_database_available_multiple_missing_password_name() {
        setup_complete_env();
        unsafe {
            env::remove_var("DB_PASSWORD");
            env::remove_var("DB_NAME");
        }

        let (available, missing) = is_database_available();

        assert!(
            !available,
            "Database should not be available when DB_PASSWORD and DB_NAME are missing"
        );
        assert_eq!(missing.len(), 2, "Should report 2 missing variables");
        assert!(
            missing.contains(&"DB_PASSWORD".to_string()),
            "DB_PASSWORD should be in missing list"
        );
        assert!(
            missing.contains(&"DB_NAME".to_string()),
            "DB_NAME should be in missing list"
        );

        setup_clean_env();
    }

    #[test]
    fn test_is_database_available_three_missing() {
        setup_complete_env();
        unsafe {
            env::remove_var("DB_HOST");
            env::remove_var("DB_USER");
            env::remove_var("DB_PASSWORD");
        }

        let (available, missing) = is_database_available();

        assert!(
            !available,
            "Database should not be available when 3 variables are missing"
        );
        assert_eq!(missing.len(), 3, "Should report 3 missing variables");
        assert!(
            missing.contains(&"DB_HOST".to_string()),
            "DB_HOST should be in missing list"
        );
        assert!(
            missing.contains(&"DB_USER".to_string()),
            "DB_USER should be in missing list"
        );
        assert!(
            missing.contains(&"DB_PASSWORD".to_string()),
            "DB_PASSWORD should be in missing list"
        );

        setup_clean_env();
    }

    #[test]
    fn test_is_database_available_db_port_optional() {
        setup_complete_env();
        unsafe {
            env::remove_var("DB_PORT");
        }

        let (available, missing) = is_database_available();

        // DB_PORTは省略可能なので、他の環境変数が設定されていれば利用可能
        assert!(
            available,
            "Database should be available even when DB_PORT is missing"
        );
        assert!(
            missing.is_empty(),
            "No environment variables should be missing when only DB_PORT is absent"
        );

        setup_clean_env();
    }

    #[test]
    fn test_is_database_available() {
        let (available, missing) = is_database_available();

        // 利用可能かどうかに関わらず、この関数は正常に実行される
        if available {
            assert!(
                missing.is_empty(),
                "If database is available, there should be no missing variables"
            );
        } else {
            assert!(
                !missing.is_empty(),
                "If database is not available, there should be missing variables"
            );
            println!("Missing database variables: {:?}", missing);
        }
    }
}
