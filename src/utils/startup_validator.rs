/// 起動時バリデーション機能
///
/// アプリケーション起動時に環境変数とファイルをチェックし、
/// 問題があれば分かりやすく表示します
use std::path::Path;
use thiserror::Error;

/// バリデーションカテゴリ
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationCategory {
    RequiredEnvVar,
    OptionalEnvVar,
    FileValidation,
}

impl ValidationCategory {
    pub fn display_name(&self) -> &str {
        match self {
            ValidationCategory::RequiredEnvVar => "Required Environment Variables",
            ValidationCategory::OptionalEnvVar => "Optional Environment Variables",
            ValidationCategory::FileValidation => "File Validation",
        }
    }
}

/// バリデーションステータス
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationStatus {
    Ok,
    Warning,
    Error,
}

impl ValidationStatus {
    pub fn symbol(&self) -> &str {
        match self {
            ValidationStatus::Ok => "✅",
            ValidationStatus::Warning => "⚠️",
            ValidationStatus::Error => "❌",
        }
    }
}

/// バリデーション結果
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub category: ValidationCategory,
    pub item_name: String,
    pub status: ValidationStatus,
    pub message: Option<String>,
    pub help_text: Option<String>,
}

impl ValidationResult {
    pub fn new(category: ValidationCategory, item_name: String, status: ValidationStatus) -> Self {
        Self {
            category,
            item_name,
            status,
            message: None,
            help_text: None,
        }
    }

    pub fn with_message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }

    pub fn with_help(mut self, help_text: String) -> Self {
        self.help_text = Some(help_text);
        self
    }
}

/// 起動時エラー
#[derive(Error, Debug)]
pub enum StartupError {
    #[error("必須環境変数が設定されていません: {var_name}")]
    MissingRequiredEnvVar { var_name: String },

    #[error("ファイルが見つかりません: {file_path} (環境変数: {env_var})")]
    FileNotFound { file_path: String, env_var: String },

    #[error("無効なJSON形式: {file_path}\n{details}")]
    InvalidJson { file_path: String, details: String },

    #[error("複数のバリデーションエラーが発生しました:\n{}", errors.join("\n"))]
    MultipleErrors { errors: Vec<String> },
}

/// 起動時バリデーションモード
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupValidationMode {
    /// 通常起動（マイグレーション + Bot起動）
    NormalStartup,
    /// マイグレーションのみ実行
    MigrationOnly,
}

/// 環境変数バリデーター
pub struct EnvValidator;

impl EnvValidator {
    /// 共通で必須の環境変数をチェック
    pub fn check_common_required_vars() -> Vec<ValidationResult> {
        let mut results = Vec::new();

        // DB_HOST
        results.push(Self::check_env_var::<fn(&str) -> Result<(), String>>(
            "DB_HOST",
            ValidationCategory::RequiredEnvVar,
            true,
            None,
        ));

        // DB_PORT
        results.push(Self::check_env_var(
            "DB_PORT",
            ValidationCategory::RequiredEnvVar,
            true,
            Some(|val: &str| {
                val.parse::<u16>()
                    .map(|_| ())
                    .map_err(|_| "有効なポート番号（1-65535）である必要があります".to_string())
            }),
        ));

        // DB_NAME
        results.push(Self::check_env_var::<fn(&str) -> Result<(), String>>(
            "DB_NAME",
            ValidationCategory::RequiredEnvVar,
            true,
            None,
        ));

        results
    }

    /// 通常起動でのみ必須の環境変数をチェック
    pub fn check_bot_startup_required_vars() -> Vec<ValidationResult> {
        let mut results = Vec::new();

        // DISCORD_TOKEN
        results.push(Self::check_env_var(
            "DISCORD_TOKEN",
            ValidationCategory::RequiredEnvVar,
            true,
            Some(|val: &str| {
                if val.len() < 30 {
                    Err("トークンが短すぎます（30文字以上必要）".to_string())
                } else {
                    Ok(())
                }
            }),
        ));

        // システムロール用（必須）
        results.push(Self::check_env_var::<fn(&str) -> Result<(), String>>(
            "SYSTEM_DB_USER",
            ValidationCategory::RequiredEnvVar,
            true,
            None,
        ));
        results.push(Self::check_env_var::<fn(&str) -> Result<(), String>>(
            "SYSTEM_DB_PASSWORD",
            ValidationCategory::RequiredEnvVar,
            true,
            None,
        ));

        // ギルドロール用（必須）
        results.push(Self::check_env_var::<fn(&str) -> Result<(), String>>(
            "GUILD_DB_USER",
            ValidationCategory::RequiredEnvVar,
            true,
            None,
        ));
        results.push(Self::check_env_var::<fn(&str) -> Result<(), String>>(
            "GUILD_DB_PASSWORD",
            ValidationCategory::RequiredEnvVar,
            true,
            None,
        ));

        // グローバルロール用（必須）
        results.push(Self::check_env_var::<fn(&str) -> Result<(), String>>(
            "GLOBAL_DB_USER",
            ValidationCategory::RequiredEnvVar,
            true,
            None,
        ));
        results.push(Self::check_env_var::<fn(&str) -> Result<(), String>>(
            "GLOBAL_DB_PASSWORD",
            ValidationCategory::RequiredEnvVar,
            true,
            None,
        ));

        // BOT_ADMIN_SERVER_ID
        results.push(Self::check_env_var(
            "BOT_ADMIN_SERVER_ID",
            ValidationCategory::RequiredEnvVar,
            true,
            Some(|val: &str| {
                if val.parse::<u64>().is_err() {
                    Err("数値である必要があります".to_string())
                } else if val.len() < 17 || val.len() > 20 {
                    Err("DiscordサーバーIDは17〜20桁です".to_string())
                } else {
                    Ok(())
                }
            }),
        ));

        // GUILD_SPREADSHEET_TEMPLATE_URL
        results.push(Self::check_env_var(
            "GUILD_SPREADSHEET_TEMPLATE_URL",
            ValidationCategory::RequiredEnvVar,
            true,
            Some(Self::validate_google_spreadsheet_template_url),
        ));

        results
    }

    /// マイグレーション実行時に必須の環境変数をチェック
    pub fn check_migration_required_vars() -> Vec<ValidationResult> {
        vec![
            // 管理者ロール用（必須）
            Self::check_env_var::<fn(&str) -> Result<(), String>>(
                "ADMIN_DB_USER",
                ValidationCategory::RequiredEnvVar,
                true,
                None,
            ),
            Self::check_env_var::<fn(&str) -> Result<(), String>>(
                "ADMIN_DB_PASSWORD",
                ValidationCategory::RequiredEnvVar,
                true,
                None,
            ),
        ]
    }

    /// 任意環境変数をチェック
    pub fn check_optional_vars() -> Vec<ValidationResult> {
        let mut results = Vec::new();

        // GLOBAL_SPREADSHEET_ID
        results.push(Self::check_env_var(
            "GLOBAL_SPREADSHEET_ID",
            ValidationCategory::OptionalEnvVar,
            false,
            Some(|val: &str| {
                if val.len() < 20 || val.len() > 80 {
                    Err("スプレッドシートIDは20〜80文字です".to_string())
                } else {
                    Ok(())
                }
            }),
        ));

        // GOOGLE_SERVICE_ACCOUNT_KEY_FILE
        results.push(Self::check_env_var::<fn(&str) -> Result<(), String>>(
            "GOOGLE_SERVICE_ACCOUNT_KEY_FILE",
            ValidationCategory::OptionalEnvVar,
            false,
            None,
        ));

        results
    }

    /// ギルドスプレッドシートテンプレートURLの妥当性を検証
    fn validate_google_spreadsheet_template_url(value: &str) -> Result<(), String> {
        let trimmed = value.trim();

        if !trimmed.starts_with("https://docs.google.com/spreadsheets/") {
            return Err("有効なGoogleスプレッドシートURLを指定してください".to_string());
        }

        if !trimmed.contains("/d/") {
            return Err(
                "GoogleスプレッドシートURLは /spreadsheets/d/{id} 形式で指定してください"
                    .to_string(),
            );
        }

        Ok(())
    }

    /// ファイル存在・内容をチェック
    pub async fn check_files() -> Vec<ValidationResult> {
        let mut results = Vec::new();

        // Googleサービスアカウントキーファイル
        if let Ok(key_file_path) = std::env::var("GOOGLE_SERVICE_ACCOUNT_KEY_FILE")
            && !key_file_path.is_empty()
        {
            let result = Self::validate_service_account_key(&key_file_path).await;
            results.push(result);
        }

        results
    }

    /// 環境変数をチェック（ヘルパー）
    fn check_env_var<F>(
        var_name: &str,
        category: ValidationCategory,
        required: bool,
        validator: Option<F>,
    ) -> ValidationResult
    where
        F: Fn(&str) -> Result<(), String>,
    {
        match std::env::var(var_name) {
            Ok(value) if !value.is_empty() => {
                // 値が存在する場合、バリデーターを実行
                if let Some(validate_fn) = validator {
                    match validate_fn(&value) {
                        Ok(()) => ValidationResult::new(
                            category,
                            var_name.to_string(),
                            ValidationStatus::Ok,
                        ),
                        Err(err_msg) => ValidationResult::new(
                            category,
                            var_name.to_string(),
                            ValidationStatus::Error,
                        )
                        .with_message(err_msg),
                    }
                } else {
                    ValidationResult::new(category, var_name.to_string(), ValidationStatus::Ok)
                }
            }
            Ok(_) | Err(_) => {
                // 値が空または存在しない
                if required {
                    ValidationResult::new(category, var_name.to_string(), ValidationStatus::Error)
                        .with_message("設定されていません".to_string())
                } else {
                    ValidationResult::new(category, var_name.to_string(), ValidationStatus::Warning)
                        .with_message(
                            "NOT SET (スプレッドシート機能を使用する場合は必須)".to_string(),
                        )
                }
            }
        }
    }

    /// サービスアカウントキーファイルをバリデート
    async fn validate_service_account_key(file_path: &str) -> ValidationResult {
        let path = Path::new(file_path);

        // ファイル存在チェック
        if !path.exists() {
            return ValidationResult::new(
                ValidationCategory::FileValidation,
                "Service Account Key File".to_string(),
                ValidationStatus::Error,
            )
            .with_message(format!("ファイルが見つかりません: {file_path}"))
            .with_help("GOOGLE_SERVICE_ACCOUNT_KEY_FILE のパスを確認してください".to_string());
        }

        // ファイル読み込み
        let content = match tokio::fs::read_to_string(file_path).await {
            Ok(c) => c,
            Err(e) => {
                return ValidationResult::new(
                    ValidationCategory::FileValidation,
                    "Service Account Key File".to_string(),
                    ValidationStatus::Error,
                )
                .with_message(format!("ファイルの読み込みに失敗しました: {e}"))
                .with_help("ファイルの権限を確認してください".to_string());
            }
        };

        // JSON形式チェック
        match serde_json::from_str::<serde_json::Value>(&content) {
            Ok(json) => {
                // 必須フィールドのチェック
                let required_fields = ["type", "project_id", "private_key", "client_email"];
                let missing_fields: Vec<_> = required_fields
                    .iter()
                    .filter(|&field| json.get(field).is_none())
                    .collect();

                if !missing_fields.is_empty() {
                    ValidationResult::new(
                        ValidationCategory::FileValidation,
                        "Service Account Key File".to_string(),
                        ValidationStatus::Error,
                    )
                    .with_message(format!(
                        "必須フィールドが不足しています: {}",
                        missing_fields
                            .iter()
                            .map(|f| f.to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ))
                    .with_help(
                        "Google Cloud Consoleから正しいサービスアカウントキーをダウンロードしてください"
                            .to_string(),
                    )
                } else {
                    ValidationResult::new(
                        ValidationCategory::FileValidation,
                        "Service Account Key File".to_string(),
                        ValidationStatus::Ok,
                    )
                    .with_message("valid JSON".to_string())
                }
            }
            Err(e) => ValidationResult::new(
                ValidationCategory::FileValidation,
                "Service Account Key File".to_string(),
                ValidationStatus::Error,
            )
            .with_message(format!("無効なJSON形式: {e}"))
            .with_help(format!("ファイル内容を確認してください: cat {file_path}")),
        }
    }
}

/// 起動時バリデーター
pub struct StartupValidator {
    results: Vec<ValidationResult>,
}

impl StartupValidator {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    /// モードに応じたチェックを実行
    pub async fn validate_for_mode(mode: StartupValidationMode) -> Result<Self, StartupError> {
        let mut validator = Self::new();

        // 共通必須環境変数チェック
        validator
            .results
            .extend(EnvValidator::check_common_required_vars());

        match mode {
            StartupValidationMode::NormalStartup => {
                // 通常起動のみ必須
                validator
                    .results
                    .extend(EnvValidator::check_bot_startup_required_vars());

                // 通常起動でもマイグレーションは実行するため必須
                validator
                    .results
                    .extend(EnvValidator::check_migration_required_vars());

                // 任意環境変数チェック
                validator
                    .results
                    .extend(EnvValidator::check_optional_vars());

                // ファイルチェック
                validator.results.extend(EnvValidator::check_files().await);
            }
            StartupValidationMode::MigrationOnly => {
                // migrate-only はマイグレーションに必要な項目のみチェック
                validator
                    .results
                    .extend(EnvValidator::check_migration_required_vars());
            }
        }

        // エラーがある場合は失敗
        if !validator.is_valid() {
            let errors: Vec<String> = validator
                .results
                .iter()
                .filter(|r| r.status == ValidationStatus::Error)
                .map(|r| {
                    format!(
                        "{}: {}",
                        r.item_name,
                        r.message.as_deref().unwrap_or("エラー")
                    )
                })
                .collect();

            return Err(StartupError::MultipleErrors { errors });
        }

        Ok(validator)
    }

    /// 通常起動向けの全チェックを実行
    pub async fn validate_all() -> Result<Self, StartupError> {
        Self::validate_for_mode(StartupValidationMode::NormalStartup).await
    }

    /// バリデーション成功か
    pub fn is_valid(&self) -> bool {
        !self
            .results
            .iter()
            .any(|r| r.status == ValidationStatus::Error)
    }

    /// 結果を表示
    pub fn display_results(&self) {
        println!("\n🔍 Starting environment validation...");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        // カテゴリごとにグループ化して表示
        for category in [
            ValidationCategory::RequiredEnvVar,
            ValidationCategory::OptionalEnvVar,
            ValidationCategory::FileValidation,
        ] {
            let category_results: Vec<_> = self
                .results
                .iter()
                .filter(|r| r.category == category)
                .collect();

            if !category_results.is_empty() {
                println!("\n{}:", category.display_name());
                for result in category_results {
                    self.display_result(result);
                }
            }
        }

        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        // サマリー表示
        let error_count = self
            .results
            .iter()
            .filter(|r| r.status == ValidationStatus::Error)
            .count();
        let warning_count = self
            .results
            .iter()
            .filter(|r| r.status == ValidationStatus::Warning)
            .count();

        if self.is_valid() {
            println!("Validation Result: ✅ PASSED");
            if warning_count > 0 {
                println!(
                    "  ({} warning{})",
                    warning_count,
                    if warning_count > 1 { "s" } else { "" }
                );
            }
            println!("\nProceeding with application startup...");
        } else {
            println!(
                "Validation Result: ❌ FAILED ({} error{}, {} warning{})",
                error_count,
                if error_count > 1 { "s" } else { "" },
                warning_count,
                if warning_count > 1 { "s" } else { "" }
            );

            // エラー詳細
            println!("\n❌ Errors:");
            for result in self
                .results
                .iter()
                .filter(|r| r.status == ValidationStatus::Error)
            {
                if let Some(msg) = &result.message {
                    println!("  - {}: {}", result.item_name, msg);
                }
            }

            // 警告詳細
            if warning_count > 0 {
                println!("\n⚠️  Warnings:");
                for result in self
                    .results
                    .iter()
                    .filter(|r| r.status == ValidationStatus::Warning)
                {
                    if let Some(msg) = &result.message {
                        println!("  - {}: {}", result.item_name, msg);
                    }
                }
            }

            // ヘルプテキスト
            let help_results: Vec<_> = self
                .results
                .iter()
                .filter(|r| r.status == ValidationStatus::Error && r.help_text.is_some())
                .collect();

            if !help_results.is_empty() {
                println!("\n💡 Next Steps:");
                for (i, result) in help_results.iter().enumerate() {
                    if let Some(help) = &result.help_text {
                        println!("  {}. {}", i + 1, help);
                    }
                }
            }

            println!("\nExiting...");
        }
    }

    /// 個別結果を表示
    fn display_result(&self, result: &ValidationResult) {
        let status_symbol = result.status.symbol();
        let padding = 40 - result.item_name.len();
        let dots = ".".repeat(padding.max(1));

        let msg = match &result.message {
            Some(m) if result.status != ValidationStatus::Ok => format!(" ({m})"),
            Some(m) => format!(" ({m})"),
            None => String::new(),
        };

        println!(
            "  {}{}{} {}{}",
            result.item_name, dots, status_symbol, status_symbol, msg
        );
    }
}

impl Default for StartupValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_status_symbol() {
        assert_eq!(ValidationStatus::Ok.symbol(), "✅");
        assert_eq!(ValidationStatus::Warning.symbol(), "⚠️");
        assert_eq!(ValidationStatus::Error.symbol(), "❌");
    }

    #[test]
    fn test_validation_result_builder() {
        let result = ValidationResult::new(
            ValidationCategory::RequiredEnvVar,
            "TEST_VAR".to_string(),
            ValidationStatus::Ok,
        )
        .with_message("テストメッセージ".to_string())
        .with_help("ヘルプテキスト".to_string());

        assert_eq!(result.item_name, "TEST_VAR");
        assert_eq!(result.status, ValidationStatus::Ok);
        assert_eq!(result.message, Some("テストメッセージ".to_string()));
        assert_eq!(result.help_text, Some("ヘルプテキスト".to_string()));
    }

    #[test]
    fn test_check_env_var_exists_and_valid() {
        unsafe {
            std::env::set_var("TEST_DISCORD_TOKEN_VALID", "a".repeat(50));
        }

        let result = EnvValidator::check_env_var(
            "TEST_DISCORD_TOKEN_VALID",
            ValidationCategory::RequiredEnvVar,
            true,
            Some(|val: &str| {
                if val.len() < 30 {
                    Err("トークンが短すぎます".to_string())
                } else {
                    Ok(())
                }
            }),
        );

        assert_eq!(result.status, ValidationStatus::Ok);
        assert_eq!(result.item_name, "TEST_DISCORD_TOKEN_VALID");

        unsafe {
            std::env::remove_var("TEST_DISCORD_TOKEN_VALID");
        }
    }

    #[test]
    fn test_check_env_var_exists_but_invalid() {
        unsafe {
            std::env::set_var("TEST_DISCORD_TOKEN_INVALID", "short");
        }

        let result = EnvValidator::check_env_var(
            "TEST_DISCORD_TOKEN_INVALID",
            ValidationCategory::RequiredEnvVar,
            true,
            Some(|val: &str| {
                if val.len() < 30 {
                    Err("トークンが短すぎます".to_string())
                } else {
                    Ok(())
                }
            }),
        );

        assert_eq!(result.status, ValidationStatus::Error);
        assert!(result.message.is_some());
        assert!(result.message.unwrap().contains("トークンが短すぎます"));

        unsafe {
            std::env::remove_var("TEST_DISCORD_TOKEN_INVALID");
        }
    }

    #[test]
    fn test_check_env_var_missing_required() {
        unsafe {
            std::env::remove_var("TEST_MISSING_VAR");
        }

        let result = EnvValidator::check_env_var::<fn(&str) -> Result<(), String>>(
            "TEST_MISSING_VAR",
            ValidationCategory::RequiredEnvVar,
            true,
            None,
        );

        assert_eq!(result.status, ValidationStatus::Error);
        assert_eq!(result.item_name, "TEST_MISSING_VAR");
        assert!(result.message.is_some());
        assert!(result.message.unwrap().contains("設定されていません"));
    }

    #[test]
    fn test_check_env_var_missing_optional() {
        unsafe {
            std::env::remove_var("TEST_OPTIONAL_VAR");
        }

        let result = EnvValidator::check_env_var::<fn(&str) -> Result<(), String>>(
            "TEST_OPTIONAL_VAR",
            ValidationCategory::OptionalEnvVar,
            false,
            None,
        );

        assert_eq!(result.status, ValidationStatus::Warning);
        assert_eq!(result.item_name, "TEST_OPTIONAL_VAR");
        assert!(result.message.is_some());
    }

    #[test]
    fn test_validate_google_spreadsheet_template_url_valid() {
        let result = EnvValidator::validate_google_spreadsheet_template_url(
            "https://docs.google.com/spreadsheets/d/1BxiMVs0XRA5nFMdKvBdBZjgmUUqptlbs74OgvE2upms/copy",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_google_spreadsheet_template_url_invalid_domain() {
        let result = EnvValidator::validate_google_spreadsheet_template_url(
            "https://example.com/spreadsheets/d/abc/copy",
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_google_spreadsheet_template_url_missing_id_segment() {
        let result = EnvValidator::validate_google_spreadsheet_template_url(
            "https://docs.google.com/spreadsheets/u/0/",
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_check_bot_startup_required_vars_includes_guild_spreadsheet_template_url() {
        let bot_required_results = EnvValidator::check_bot_startup_required_vars();
        let template_url_result = bot_required_results
            .iter()
            .find(|result| result.item_name == "GUILD_SPREADSHEET_TEMPLATE_URL");

        assert!(template_url_result.is_some());
        assert_eq!(
            template_url_result.unwrap().category,
            ValidationCategory::RequiredEnvVar
        );
    }

    #[test]
    fn test_check_migration_required_vars_excludes_guild_spreadsheet_template_url() {
        let migration_required_results = EnvValidator::check_migration_required_vars();
        let template_url_result = migration_required_results
            .iter()
            .find(|result| result.item_name == "GUILD_SPREADSHEET_TEMPLATE_URL");

        assert!(template_url_result.is_none());
    }

    #[test]
    fn test_check_common_required_vars_excludes_discord_token() {
        let common_required_results = EnvValidator::check_common_required_vars();
        let discord_token_result = common_required_results
            .iter()
            .find(|result| result.item_name == "DISCORD_TOKEN");

        assert!(discord_token_result.is_none());
    }

    #[test]
    fn test_check_migration_required_vars_includes_admin_db_user() {
        let migration_required_results = EnvValidator::check_migration_required_vars();
        let admin_user_result = migration_required_results
            .iter()
            .find(|result| result.item_name == "ADMIN_DB_USER");

        assert!(admin_user_result.is_some());
        assert_eq!(
            admin_user_result.unwrap().category,
            ValidationCategory::RequiredEnvVar
        );
    }

    #[tokio::test]
    async fn test_validate_service_account_key_valid_json() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // 有効なJSONファイルを作成
        let mut temp_file = NamedTempFile::new().unwrap();
        let valid_json = r#"{
            "type": "service_account",
            "project_id": "test-project",
            "private_key": "-----BEGIN PRIVATE KEY-----\ntest\n-----END PRIVATE KEY-----",
            "client_email": "test@test-project.iam.gserviceaccount.com"
        }"#;
        temp_file.write_all(valid_json.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let result =
            EnvValidator::validate_service_account_key(temp_file.path().to_str().unwrap()).await;

        assert_eq!(result.status, ValidationStatus::Ok);
        assert!(result.message.is_some());
        assert!(result.message.unwrap().contains("valid JSON"));
    }

    #[tokio::test]
    async fn test_validate_service_account_key_invalid_json() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // 無効なJSONファイルを作成
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"{ invalid json }").unwrap();
        temp_file.flush().unwrap();

        let result =
            EnvValidator::validate_service_account_key(temp_file.path().to_str().unwrap()).await;

        assert_eq!(result.status, ValidationStatus::Error);
        assert!(result.message.is_some());
        assert!(result.message.unwrap().contains("無効なJSON形式"));
    }

    #[tokio::test]
    async fn test_validate_service_account_key_missing_fields() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // 必須フィールドが欠けているJSONファイルを作成
        let mut temp_file = NamedTempFile::new().unwrap();
        let incomplete_json = r#"{
            "type": "service_account",
            "project_id": "test-project"
        }"#;
        temp_file.write_all(incomplete_json.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let result =
            EnvValidator::validate_service_account_key(temp_file.path().to_str().unwrap()).await;

        assert_eq!(result.status, ValidationStatus::Error);
        assert!(result.message.is_some());
        let msg = result.message.unwrap();
        assert!(msg.contains("必須フィールドが不足"));
        assert!(msg.contains("private_key") || msg.contains("client_email"));
    }

    #[tokio::test]
    async fn test_validate_service_account_key_file_not_found() {
        let result =
            EnvValidator::validate_service_account_key("/nonexistent/path/to/keyfile.json").await;

        assert_eq!(result.status, ValidationStatus::Error);
        assert!(result.message.is_some());
        assert!(result.message.unwrap().contains("ファイルが見つかりません"));
    }

    #[test]
    fn test_startup_validator_is_valid_all_ok() {
        let validator = StartupValidator {
            results: vec![
                ValidationResult::new(
                    ValidationCategory::RequiredEnvVar,
                    "VAR1".to_string(),
                    ValidationStatus::Ok,
                ),
                ValidationResult::new(
                    ValidationCategory::RequiredEnvVar,
                    "VAR2".to_string(),
                    ValidationStatus::Ok,
                ),
            ],
        };

        assert!(validator.is_valid());
    }

    #[test]
    fn test_startup_validator_is_valid_with_warnings() {
        let validator = StartupValidator {
            results: vec![
                ValidationResult::new(
                    ValidationCategory::RequiredEnvVar,
                    "VAR1".to_string(),
                    ValidationStatus::Ok,
                ),
                ValidationResult::new(
                    ValidationCategory::OptionalEnvVar,
                    "VAR2".to_string(),
                    ValidationStatus::Warning,
                ),
            ],
        };

        // Warningだけならvalidと判定される
        assert!(validator.is_valid());
    }

    #[test]
    fn test_startup_validator_is_invalid_with_error() {
        let validator = StartupValidator {
            results: vec![
                ValidationResult::new(
                    ValidationCategory::RequiredEnvVar,
                    "VAR1".to_string(),
                    ValidationStatus::Ok,
                ),
                ValidationResult::new(
                    ValidationCategory::RequiredEnvVar,
                    "VAR2".to_string(),
                    ValidationStatus::Error,
                )
                .with_message("エラーが発生しました".to_string()),
            ],
        };

        assert!(!validator.is_valid());
    }

    #[test]
    fn test_startup_error_display() {
        let error = StartupError::MultipleErrors {
            errors: vec![
                "環境変数1が設定されていません".to_string(),
                "環境変数2が不正です".to_string(),
            ],
        };

        let error_str = format!("{error}");
        assert!(error_str.contains("複数のバリデーションエラーが発生しました"));
    }
}
