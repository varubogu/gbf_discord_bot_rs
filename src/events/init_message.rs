use crate::events::helpers::resolve_guild_locale;
use crate::services::message::MessageTextId;
use crate::types::AppState;
use serde::Deserialize;
use std::collections::HashMap;
use tracing::{debug, warn};

const TEMPLATE_URL_ENV_KEY: &str = "GUILD_SPREADSHEET_TEMPLATE_URL";
const SERVICE_ACCOUNT_KEY_FILE_ENV_KEY: &str = "GOOGLE_SERVICE_ACCOUNT_KEY_FILE";
const UNAVAILABLE_VALUE: &str = "N/A";

#[derive(Debug, Deserialize)]
struct ServiceAccountKeyPreview {
    client_email: Option<String>,
}

/// 初期設定メッセージ本文を生成する
///
/// `locales/messages.yml` の `messages.init_guide` を取得し、
/// テンプレートURLとサービスアカウントメールを埋め込んで返す。
pub async fn build_init_guide_message(app_state: &AppState, guild_id: i64) -> String {
    let locale = resolve_guild_locale(app_state, Some(guild_id)).await;
    let template_url = resolve_template_url();
    let service_account_email = resolve_service_account_email().await;

    let mut params = HashMap::new();
    params.insert("template_url".to_string(), template_url);
    params.insert("service_account_email".to_string(), service_account_email);

    match app_state
        .message_service()
        .get_message(
            app_state.guild_db(),
            MessageTextId::MessagesInitGuide.as_str(),
            params,
            Some(guild_id),
            Some(&locale),
        )
        .await
    {
        Ok(message) => message,
        Err(e) => {
            warn!(
                error = %e,
                guild_id = guild_id,
                "初期設定メッセージの取得に失敗したためメッセージキーを返します"
            );
            MessageTextId::MessagesInitGuide.as_str().to_string()
        }
    }
}

fn resolve_template_url() -> String {
    resolve_template_url_from_env(std::env::var(TEMPLATE_URL_ENV_KEY).ok())
}

fn resolve_template_url_from_env(env_value: Option<String>) -> String {
    let Some(value) = env_value else {
        return UNAVAILABLE_VALUE.to_string();
    };
    let trimmed = value.trim();
    if trimmed.is_empty() {
        UNAVAILABLE_VALUE.to_string()
    } else {
        trimmed.to_string()
    }
}

async fn resolve_service_account_email() -> String {
    let key_file_path = match std::env::var(SERVICE_ACCOUNT_KEY_FILE_ENV_KEY) {
        Ok(path) if !path.trim().is_empty() => path,
        _ => {
            debug!("サービスアカウントキーファイルの環境変数が未設定です");
            return UNAVAILABLE_VALUE.to_string();
        }
    };

    let key_file_content = match tokio::fs::read_to_string(&key_file_path).await {
        Ok(content) => content,
        Err(e) => {
            warn!(
                error = %e,
                path = %key_file_path,
                "サービスアカウントキーファイルの読み込みに失敗しました"
            );
            return UNAVAILABLE_VALUE.to_string();
        }
    };

    match extract_service_account_email_from_json(&key_file_content) {
        Some(email) => email,
        None => {
            warn!(
                path = %key_file_path,
                "サービスアカウントメールの抽出に失敗しました"
            );
            UNAVAILABLE_VALUE.to_string()
        }
    }
}

fn extract_service_account_email_from_json(json_content: &str) -> Option<String> {
    let key: ServiceAccountKeyPreview = serde_json::from_str(json_content).ok()?;
    let client_email = key.client_email?;
    let trimmed = client_email.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_template_url_from_env_uses_value() {
        let resolved = resolve_template_url_from_env(Some(
            "https://docs.google.com/spreadsheets/d/template-id/copy".to_string(),
        ));
        assert_eq!(
            resolved,
            "https://docs.google.com/spreadsheets/d/template-id/copy"
        );
    }

    #[test]
    fn test_resolve_template_url_from_env_returns_unavailable_when_empty() {
        let resolved = resolve_template_url_from_env(Some("   ".to_string()));
        assert_eq!(resolved, UNAVAILABLE_VALUE);
    }

    #[test]
    fn test_resolve_template_url_from_env_returns_unavailable_when_none() {
        let resolved = resolve_template_url_from_env(None);
        assert_eq!(resolved, UNAVAILABLE_VALUE);
    }

    #[test]
    fn test_extract_service_account_email_from_json_success() {
        let json = r#"{"client_email":"bot-service@example.iam.gserviceaccount.com"}"#;
        let email = extract_service_account_email_from_json(json);
        assert_eq!(
            email,
            Some("bot-service@example.iam.gserviceaccount.com".to_string())
        );
    }

    #[test]
    fn test_extract_service_account_email_from_json_returns_none_for_missing_field() {
        let json = r#"{"type":"service_account"}"#;
        let email = extract_service_account_email_from_json(json);
        assert_eq!(email, None);
    }

    #[test]
    fn test_extract_service_account_email_from_json_returns_none_for_invalid_json() {
        let json = r#"{"client_email":"x@example.com""#;
        let email = extract_service_account_email_from_json(json);
        assert_eq!(email, None);
    }
}
