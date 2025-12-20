/// エラーフォーマッター
///
/// 実行時エラーを分かりやすく整形して表示します
use sea_orm::DbErr;
use std::error::Error;

/// エラーフォーマッター
pub struct ErrorFormatter;

impl ErrorFormatter {
    /// データベースエラーをフォーマット
    pub fn format_db_error(err: &DbErr, db_url_masked: &str) -> String {
        let error_type = match err {
            DbErr::Conn(_) => "Database Connection Error",
            DbErr::Exec(_) => "Database Execution Error",
            DbErr::Query(_) => "Database Query Error",
            _ => "Database Error",
        };

        let error_details = err.to_string();

        let possible_causes = ["PostgreSQLサーバーが起動していない",
            "データベース名が間違っている",
            "ホストまたはポートに到達できない",
            "認証に失敗している"];

        let troubleshooting = ["PostgreSQLサーバーの状態を確認: sudo systemctl status postgresql",
            "データベース接続情報を確認: DB_HOST, DB_PORT, DB_NAME, *_DB_USER, *_DB_PASSWORD",
            "接続をテスト: psql \"postgresql://<user>:<password>@<host>:<port>/<database>\""];

        format!(
            "\n❌ {}\n\
            \n\
            Database Connection: {}\n\
            Error Details: {}\n\
            \n\
            Possible Causes:\n{}\n\
            \n\
            💡 Troubleshooting:\n{}\n",
            error_type,
            db_url_masked,
            error_details,
            possible_causes
                .iter()
                .map(|c| format!("  - {c}"))
                .collect::<Vec<_>>()
                .join("\n"),
            troubleshooting
                .iter()
                .enumerate()
                .map(|(i, t)| format!("  {}. {}", i + 1, t))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }

    /// JSONパースエラーをフォーマット
    pub fn format_json_error(err: &serde_json::Error, file_path: &str) -> String {
        let error_details = err.to_string();

        let possible_causes = ["ファイルが空である",
            "無効なJSON形式が含まれている",
            "ファイルのエンコーディングが不正"];

        let troubleshooting = [format!("ファイル内容を確認: cat {file_path}"),
            format!("JSON形式を検証: jq . {file_path}"),
            "Google Cloud Consoleからサービスアカウントキーを再ダウンロード".to_string()];

        format!(
            "\n❌ JSON Parse Error\n\
            \n\
            Environment Variable: GOOGLE_SERVICE_ACCOUNT_KEY_FILE\n\
            File Path: {}\n\
            Error Details: {}\n\
            \n\
            Possible Causes:\n{}\n\
            \n\
            💡 Troubleshooting:\n{}\n",
            file_path,
            error_details,
            possible_causes
                .iter()
                .map(|c| format!("  - {c}"))
                .collect::<Vec<_>>()
                .join("\n"),
            troubleshooting
                .iter()
                .enumerate()
                .map(|(i, t)| format!("  {}. {}", i + 1, t))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }

    /// 一般的なエラーをフォーマット
    pub fn format_generic_error(err: &dyn Error, context: &str) -> String {
        format!(
            "\n❌ Error: {}\n\
            \n\
            Details: {}\n\
            \n\
            Context: {}\n",
            err,
            err.source()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "詳細情報なし".to_string()),
            context
        )
    }

    /// データベースURLをマスク（パスワード部分を隠す）
    pub fn mask_database_url(db_url: &str) -> String {
        // postgresql://user:password@host:port/database の形式
        if let Some(at_pos) = db_url.rfind('@') {
            if let Some(colon_pos) = db_url[..at_pos].rfind(':') {
                // パスワード部分をマスク
                let before_password = &db_url[..colon_pos + 1];
                let after_password = &db_url[at_pos..];
                return format!("{before_password}***{after_password}");
            }
        }
        // マスクできない場合はそのまま返す（パスワードがない形式の可能性）
        db_url.to_string()
    }

    /// トークンをマスク（先頭と末尾のみ表示）
    pub fn mask_token(token: &str) -> String {
        if token.len() <= 10 {
            return "***".to_string();
        }
        format!("{}...{}", &token[..4], &token[token.len() - 4..])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_database_url() {
        let url = "postgresql://user:mypassword@localhost:5432/mydb";
        let masked = ErrorFormatter::mask_database_url(url);
        assert_eq!(masked, "postgresql://user:***@localhost:5432/mydb");
    }

    #[test]
    fn test_mask_database_url_no_password() {
        let url = "postgresql://localhost:5432/mydb";
        let masked = ErrorFormatter::mask_database_url(url);
        assert_eq!(masked, "postgresql://localhost:5432/mydb");
    }

    #[test]
    fn test_mask_token() {
        let token = "MTIzNDU2Nzg5MDEyMzQ1Njc4OTA";
        let masked = ErrorFormatter::mask_token(token);
        assert_eq!(masked, "MTIz...4OTA");
    }

    #[test]
    fn test_mask_token_short() {
        let token = "short";
        let masked = ErrorFormatter::mask_token(token);
        assert_eq!(masked, "***");
    }

    #[test]
    fn test_format_db_error_connection_refused() {
        use sea_orm::{DbErr, RuntimeErr};

        let db_err = DbErr::Conn(RuntimeErr::Internal("connection refused".to_string()));
        let masked_url = "postgresql://user:***@localhost:5432/mydb";

        let formatted = ErrorFormatter::format_db_error(&db_err, masked_url);

        assert!(formatted.contains("Database Connection Error"));
        assert!(formatted.contains("Database Connection"));
        assert!(formatted.contains(masked_url));
        assert!(formatted.contains("connection refused"));
        assert!(formatted.contains("PostgreSQLサーバーが起動していない"));
        assert!(formatted.contains("Troubleshooting"));
    }

    #[test]
    fn test_format_db_error_query_error() {
        use sea_orm::DbErr;

        let db_err = DbErr::Query(sea_orm::RuntimeErr::Internal("syntax error".to_string()));
        let masked_url = "postgresql://user:***@localhost:5432/mydb";

        let formatted = ErrorFormatter::format_db_error(&db_err, masked_url);

        assert!(formatted.contains("Database Query Error"));
        assert!(formatted.contains("Database Connection"));
        assert!(formatted.contains("syntax error"));
    }

    #[test]
    fn test_format_json_error_eof() {
        let json_err = serde_json::from_str::<serde_json::Value>("").unwrap_err();
        let file_path = "/path/to/service-account.json";

        let formatted = ErrorFormatter::format_json_error(&json_err, file_path);

        assert!(formatted.contains("JSON Parse Error"));
        assert!(formatted.contains("GOOGLE_SERVICE_ACCOUNT_KEY_FILE"));
        assert!(formatted.contains(file_path));
        assert!(formatted.contains("ファイルが空である"));
        assert!(formatted.contains("Troubleshooting"));
        assert!(formatted.contains("cat"));
    }

    #[test]
    fn test_format_json_error_invalid_syntax() {
        let json_err = serde_json::from_str::<serde_json::Value>("{ invalid }").unwrap_err();
        let file_path = "/path/to/service-account.json";

        let formatted = ErrorFormatter::format_json_error(&json_err, file_path);

        assert!(formatted.contains("JSON Parse Error"));
        assert!(formatted.contains("無効なJSON形式が含まれている"));
        assert!(formatted.contains("JSON形式を検証: jq"));
    }

    #[test]
    fn test_format_generic_error() {
        use std::fmt;

        #[derive(Debug)]
        struct TestError {
            message: String,
        }

        impl fmt::Display for TestError {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}", self.message)
            }
        }

        impl std::error::Error for TestError {}

        let err = TestError {
            message: "APIキーが無効です".to_string(),
        };
        let context = "Google認証";

        let formatted = ErrorFormatter::format_generic_error(&err, context);

        assert!(formatted.contains("Error"));
        assert!(formatted.contains("Google認証"));
        assert!(formatted.contains("APIキーが無効です"));
    }

    #[test]
    fn test_mask_database_url_with_special_chars() {
        let url = "postgresql://user:p@ss!w0rd@localhost:5432/mydb";
        let masked = ErrorFormatter::mask_database_url(url);
        assert_eq!(masked, "postgresql://user:***@localhost:5432/mydb");
    }

    #[test]
    fn test_mask_database_url_postgres_scheme() {
        let url = "postgres://user:password@localhost:5432/mydb";
        let masked = ErrorFormatter::mask_database_url(url);
        assert_eq!(masked, "postgres://user:***@localhost:5432/mydb");
    }

    #[test]
    fn test_mask_token_exactly_10_chars() {
        let token = "1234567890";
        let masked = ErrorFormatter::mask_token(token);
        assert_eq!(masked, "***");
    }

    #[test]
    fn test_mask_token_11_chars() {
        let token = "12345678901";
        let masked = ErrorFormatter::mask_token(token);
        assert_eq!(masked, "1234...8901");
    }
}
