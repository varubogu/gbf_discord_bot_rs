//! オートコンプリート関連のドメイン型
//!
//! コマンドのオートコンプリート機能で使用する型を定義する。

/// オートコンプリート選択肢
///
/// コマンドパラメータの入力補完候補として使用する。
///
/// # Example
///
/// ```
/// use gbf_discord_bot_rs::types::discord::AutocompleteOption;
///
/// // 基本的な作成方法
/// let option = AutocompleteOption::new("表示名", "値");
///
/// // タプルからの変換
/// let option: AutocompleteOption = ("名前", "value").into();
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutocompleteOption {
    /// ユーザーに表示される名前
    pub name: String,
    /// 選択時に使用される値
    pub value: String,
}

impl AutocompleteOption {
    /// 新しいオートコンプリート選択肢を作成する
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }

    /// 名前と値が同じオートコンプリート選択肢を作成する
    pub fn same(value: impl Into<String>) -> Self {
        let v = value.into();
        Self {
            name: v.clone(),
            value: v,
        }
    }
}

/// Stringタプルからの変換
impl From<(String, String)> for AutocompleteOption {
    fn from((name, value): (String, String)) -> Self {
        Self { name, value }
    }
}

/// &strタプルからの変換
impl From<(&str, &str)> for AutocompleteOption {
    fn from((name, value): (&str, &str)) -> Self {
        Self {
            name: name.to_string(),
            value: value.to_string(),
        }
    }
}

/// 単一の文字列からの変換（名前と値を同一にする）
impl From<String> for AutocompleteOption {
    fn from(value: String) -> Self {
        Self::same(value)
    }
}

/// &strからの変換（名前と値を同一にする）
impl From<&str> for AutocompleteOption {
    fn from(value: &str) -> Self {
        Self::same(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_autocomplete_option_new() {
        let option = AutocompleteOption::new("表示名", "value");
        assert_eq!(option.name, "表示名");
        assert_eq!(option.value, "value");
    }

    #[test]
    fn test_autocomplete_option_same() {
        let option = AutocompleteOption::same("同じ値");
        assert_eq!(option.name, "同じ値");
        assert_eq!(option.value, "同じ値");
    }

    #[test]
    fn test_from_string_tuple() {
        let option: AutocompleteOption = ("名前".to_string(), "値".to_string()).into();
        assert_eq!(option.name, "名前");
        assert_eq!(option.value, "値");
    }

    #[test]
    fn test_from_str_tuple() {
        let option: AutocompleteOption = ("名前", "値").into();
        assert_eq!(option.name, "名前");
        assert_eq!(option.value, "値");
    }

    #[test]
    fn test_from_string() {
        let option: AutocompleteOption = "テスト".to_string().into();
        assert_eq!(option.name, "テスト");
        assert_eq!(option.value, "テスト");
    }

    #[test]
    fn test_from_str() {
        let option: AutocompleteOption = "テスト".into();
        assert_eq!(option.name, "テスト");
        assert_eq!(option.value, "テスト");
    }
}
