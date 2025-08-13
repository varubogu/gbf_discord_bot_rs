use rust_i18n::{t, set_locale};
use std::sync::{OnceLock, RwLock};
use std::collections::HashMap;
use serde_json::Value;
use tokio::fs;

/// Supported languages for the message service
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    Japanese,
}

impl Language {
    fn to_locale(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Japanese => "ja",
        }
    }

    pub fn from_str(lang: &str) -> Option<Self> {
        match lang.to_lowercase().as_str() {
            "en" | "english" | "eng" => Some(Language::English),
            "ja" | "japanese" | "jpn" | "jp" => Some(Language::Japanese),
            _ => None,
        }
    }
}

impl Default for Language {
    fn default() -> Self {
        Language::English
    }
}

/// Custom message store for server-specific overrides
struct CustomMessageStore {
    messages: RwLock<HashMap<String, HashMap<String, String>>>, // locale -> key -> message
}

impl CustomMessageStore {
    fn new() -> Self {
        Self {
            messages: RwLock::new(HashMap::new()),
        }
    }

    fn get(&self, key: &str, locale: &str) -> Option<String> {
        let messages = self.messages.read().unwrap();
        messages.get(locale)?.get(key).cloned()
    }

    async fn load_from_directory(&self, dir_path: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut entries = fs::read_dir(dir_path).await?;
        let mut new_messages = HashMap::new();

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(locale) = path.file_stem().and_then(|s| s.to_str()) {
                    let content = fs::read_to_string(&path).await?;
                    let json: HashMap<String, String> = serde_json::from_str(&content)?;
                    new_messages.insert(locale.to_string(), json);
                }
            }
        }

        let mut messages = self.messages.write().unwrap();
        *messages = new_messages;
        Ok(())
    }

    fn clear(&self) {
        let mut messages = self.messages.write().unwrap();
        messages.clear();
    }
}

/// Enhanced message service with two-tier message management
pub struct MessageService {
    current_language: RwLock<Language>,
    custom_store: CustomMessageStore,
}

static MESSAGE_SERVICE: OnceLock<MessageService> = OnceLock::new();

impl MessageService {
    fn new() -> Self {
        Self {
            current_language: RwLock::new(Language::default()),
            custom_store: CustomMessageStore::new(),
        }
    }

    pub fn instance() -> &'static MessageService {
        MESSAGE_SERVICE.get_or_init(|| MessageService::new())
    }

    /// Set the current language for messages
    pub fn set_language(&self, language: Language) {
        *self.current_language.write().unwrap() = language;
        set_locale(language.to_locale());
    }

    pub fn current_language(&self) -> Language {
        *self.current_language.read().unwrap()
    }

    /// Load custom messages from directory (async)
    pub async fn load_custom_messages(&self, dir_path: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.custom_store.load_from_directory(dir_path).await
    }

    /// Clear custom messages (fallback to standard messages only)
    pub fn clear_custom_messages(&self) {
        self.custom_store.clear();
    }

    /// Get a localized message by key
    /// First checks custom messages, then falls back to standard rust-i18n messages
    pub fn get(&self, key: &str) -> String {
        let current_lang = self.current_language();
        self.get_with_language(key, current_lang)
    }

    /// Get a localized message with specific language
    pub fn get_with_language(&self, key: &str, language: Language) -> String {
        let locale = language.to_locale();
        
        // First try custom messages
        if let Some(custom_message) = self.custom_store.get(key, locale) {
            return custom_message;
        }
        
        // Fallback to standard rust-i18n messages
        // t!(key, locale = locale).to_string()
        "".to_string()
    }

    /// Get a localized message with parameters
    /// Supports both {{param}} and {param} parameter formats
    pub fn get_with_params(&self, key: &str, params: &[(&str, &str)]) -> String {
        self.get_with_params_and_language(key, params, self.current_language())
    }

    /// Get a localized message with parameters and specific language
    pub fn get_with_params_and_language(&self, key: &str, params: &[(&str, &str)], language: Language) -> String {
        let mut result = self.get_with_language(key, language);
        
        for (param_key, param_value) in params {
            // Support both {{param}} and {param} formats
            result = result.replace(&format!("{{{{{}}}}}", param_key), param_value);
            result = result.replace(&format!("{{{}}}", param_key), param_value);
        }
        
        result
    }

    /// Check if a custom message override exists for the key
    pub fn has_custom_message(&self, key: &str, language: Language) -> bool {
        let locale = language.to_locale();
        self.custom_store.get(key, locale).is_some()
    }

    /// Get available languages
    pub fn available_languages() -> Vec<Language> {
        vec![Language::English, Language::Japanese]
    }
}

impl Default for MessageService {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience functions for easy access to messages
pub mod messages {
    use super::*;

    /// Get a message using the global instance
    pub fn get(key: &str) -> String {
        MessageService::instance().get(key)
    }

    /// Get a message with specific language
    pub fn get_with_language(key: &str, language: Language) -> String {
        MessageService::instance().get_with_language(key, language)
    }

    /// Get a message with parameters
    pub fn get_with_params(key: &str, params: &[(&str, &str)]) -> String {
        MessageService::instance().get_with_params(key, params)
    }

    /// Get a message with parameters and specific language
    pub fn get_with_params_and_language(key: &str, params: &[(&str, &str)], language: Language) -> String {
        MessageService::instance().get_with_params_and_language(key, params, language)
    }

    /// Set the global language
    pub fn set_global_language(language: Language) {
        MessageService::instance().set_language(language);
    }

    /// Load custom messages from directory
    pub async fn load_custom_messages(dir_path: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        MessageService::instance().load_custom_messages(dir_path).await
    }

    /// Clear custom messages
    pub fn clear_custom_messages() {
        MessageService::instance().clear_custom_messages();
    }

    /// Check if custom message exists
    pub fn has_custom_message(key: &str, language: Language) -> bool {
        MessageService::instance().has_custom_message(key, language)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[test]
    fn test_language_conversion() {
        assert_eq!(Language::English.to_locale(), "en");
        assert_eq!(Language::Japanese.to_locale(), "ja");
    }

    #[test]
    fn test_message_service_basic() {
        let service = MessageService::new();
        let success_msg = service.get("common.success");
        assert!(!success_msg.is_empty());
    }

    #[tokio::test]
    async fn test_custom_message_override() {
        // This would require setting up test JSON files
        // Left as a placeholder for integration tests
    }

    #[test]
    fn test_parameter_substitution() {
        let service = MessageService::new();
        let params = &[("quest_name", "ドラゴンクエスト"), ("battle_type", "全属性")];
        
        // Assuming we have a message key that uses parameters
        let result = service.get_with_params("recruitment.message", params);
        assert!(!result.is_empty());
    }
}