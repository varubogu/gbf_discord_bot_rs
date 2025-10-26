/// SpreadsheetUrlService
///
/// GoogleスプレッドシートのURLからIDを抽出し、正規化する

use crate::errors::ValidationError;
use regex::Regex;

pub trait SpreadsheetUrlServiceTrait: Send + Sync {
    /// URLまたはIDからスプレッドシートIDを抽出
    fn extract_spreadsheet_id(&self, url_or_id: &str) -> Result<String, ValidationError>;

    /// スプレッドシートIDを検証
    fn validate_spreadsheet_id(&self, spreadsheet_id: &str) -> Result<(), ValidationError>;

    /// スプレッドシートIDから完全なURLを生成
    fn build_spreadsheet_url(&self, spreadsheet_id: &str) -> String;
}

#[derive(Clone)]
pub struct SpreadsheetUrlService {
    /// スプレッドシートURL内のIDを抽出する正規表現
    url_pattern: Regex,
    /// スプレッドシートIDの形式を検証する正規表現
    id_pattern: Regex,
}

impl SpreadsheetUrlService {
    pub fn new() -> Self {
        Self {
            // GoogleスプレッドシートのURLパターン
            // https://docs.google.com/spreadsheets/d/{id}/... の形式
            url_pattern: Regex::new(r"https://docs\.google\.com/spreadsheets/d/([A-Za-z0-9-_]+)")
                .expect("正規表現パターンが不正です"),
            // スプレッドシートIDは20〜80文字の英数字とハイフン、アンダースコア
            id_pattern: Regex::new(r"^[A-Za-z0-9-_]{20,80}$")
                .expect("正規表現パターンが不正です"),
        }
    }
}

impl Default for SpreadsheetUrlService {
    fn default() -> Self {
        Self::new()
    }
}

impl SpreadsheetUrlServiceTrait for SpreadsheetUrlService {
    fn extract_spreadsheet_id(&self, url_or_id: &str) -> Result<String, ValidationError> {
        let trimmed = url_or_id.trim();

        // URLの場合、IDを抽出
        if let Some(captures) = self.url_pattern.captures(trimmed) {
            if let Some(id_match) = captures.get(1) {
                let id = id_match.as_str().to_string();
                self.validate_spreadsheet_id(&id)?;
                return Ok(id);
            }
        }

        // ID形式の場合、そのまま検証して返す
        self.validate_spreadsheet_id(trimmed)?;
        Ok(trimmed.to_string())
    }

    fn validate_spreadsheet_id(&self, spreadsheet_id: &str) -> Result<(), ValidationError> {
        if !self.id_pattern.is_match(spreadsheet_id) {
            return Err(ValidationError::InvalidFormat {
                field: "spreadsheet_id".to_string(),
                reason: "スプレッドシートIDの形式が不正です（20〜80文字の英数字、ハイフン、アンダースコアのみ）".to_string(),
            });
        }
        Ok(())
    }

    fn build_spreadsheet_url(&self, spreadsheet_id: &str) -> String {
        format!(
            "https://docs.google.com/spreadsheets/d/{}",
            spreadsheet_id
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_from_full_url() {
        let service = SpreadsheetUrlService::new();
        let url = "https://docs.google.com/spreadsheets/d/1BxiMVs0XRA5nFMdKvBdBZjgmUUqptlbs74OgvE2upms/edit";
        let result = service.extract_spreadsheet_id(url);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "1BxiMVs0XRA5nFMdKvBdBZjgmUUqptlbs74OgvE2upms"
        );
    }

    #[test]
    fn test_extract_from_id_only() {
        let service = SpreadsheetUrlService::new();
        let id = "1BxiMVs0XRA5nFMdKvBdBZjgmUUqptlbs74OgvE2upms";
        let result = service.extract_spreadsheet_id(id);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "1BxiMVs0XRA5nFMdKvBdBZjgmUUqptlbs74OgvE2upms"
        );
    }

    #[test]
    fn test_extract_from_url_with_trailing_slash() {
        let service = SpreadsheetUrlService::new();
        let url = "https://docs.google.com/spreadsheets/d/1BxiMVs0XRA5nFMdKvBdBZjgmUUqptlbs74OgvE2upms/";
        let result = service.extract_spreadsheet_id(url);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_url_format() {
        let service = SpreadsheetUrlService::new();
        let url = "https://example.com/invalid";
        let result = service.extract_spreadsheet_id(url);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_id_too_short() {
        let service = SpreadsheetUrlService::new();
        let id = "short";
        let result = service.extract_spreadsheet_id(id);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_id_with_special_chars() {
        let service = SpreadsheetUrlService::new();
        let id = "invalid-id-with-special@chars!";
        let result = service.extract_spreadsheet_id(id);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_url() {
        let service = SpreadsheetUrlService::new();
        let id = "1BxiMVs0XRA5nFMdKvBdBZjgmUUqptlbs74OgvE2upms";
        let url = service.build_spreadsheet_url(id);
        assert_eq!(
            url,
            "https://docs.google.com/spreadsheets/d/1BxiMVs0XRA5nFMdKvBdBZjgmUUqptlbs74OgvE2upms"
        );
    }

    #[test]
    fn test_validate_valid_id() {
        let service = SpreadsheetUrlService::new();
        let id = "1BxiMVs0XRA5nFMdKvBdBZjgmUUqptlbs74OgvE2upms";
        let result = service.validate_spreadsheet_id(id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_id() {
        let service = SpreadsheetUrlService::new();
        let id = "invalid";
        let result = service.validate_spreadsheet_id(id);
        assert!(result.is_err());
    }

    #[test]
    fn test_trim_whitespace() {
        let service = SpreadsheetUrlService::new();
        let id_with_spaces = "  1BxiMVs0XRA5nFMdKvBdBZjgmUUqptlbs74OgvE2upms  ";
        let result = service.extract_spreadsheet_id(id_with_spaces);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "1BxiMVs0XRA5nFMdKvBdBZjgmUUqptlbs74OgvE2upms"
        );
    }
}
