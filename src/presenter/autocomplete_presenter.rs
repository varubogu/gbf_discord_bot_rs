//! オートコンプリートプレゼンター
//!
//! オートコンプリート候補の表示フォーマットを担当する。

use crate::services::quest::search::QuestAutocompleteItem;
use crate::types::discord::AutocompleteOption;

/// オートコンプリートの表示を担当するPresenter
///
/// Service層から取得したデータをAutocompleteOption形式に変換する。
pub struct AutocompletePresenter;

impl AutocompletePresenter {
    /// クエスト検索結果をオートコンプリートオプションに変換する
    ///
    /// # Arguments
    ///
    /// * `items` - クエスト検索サービスから取得したオートコンプリートアイテム
    ///
    /// # Returns
    ///
    /// Discord APIに渡すオートコンプリートオプションのリスト
    pub fn from_quest_items(items: Vec<QuestAutocompleteItem>) -> Vec<AutocompleteOption> {
        items
            .into_iter()
            .map(|item| AutocompleteOption::new(item.display_name, item.quest_name))
            .collect()
    }

    /// 文字列リストをオートコンプリートオプションに変換する
    ///
    /// 名前と値が同じ場合に使用する。
    ///
    /// # Arguments
    ///
    /// * `items` - 変換する文字列リスト
    ///
    /// # Returns
    ///
    /// Discord APIに渡すオートコンプリートオプションのリスト
    pub fn from_strings(items: Vec<String>) -> Vec<AutocompleteOption> {
        items.into_iter().map(AutocompleteOption::same).collect()
    }

    /// 名前と値のペアからオートコンプリートオプションに変換する
    ///
    /// # Arguments
    ///
    /// * `pairs` - (表示名, 値) のタプルリスト
    ///
    /// # Returns
    ///
    /// Discord APIに渡すオートコンプリートオプションのリスト
    pub fn from_pairs(pairs: Vec<(String, String)>) -> Vec<AutocompleteOption> {
        pairs.into_iter().map(AutocompleteOption::from).collect()
    }

    /// 部分一致でフィルタリングしたオートコンプリートオプションを返す
    ///
    /// # Arguments
    ///
    /// * `options` - フィルタ対象のオプションリスト
    /// * `partial` - 部分一致検索文字列
    /// * `max_results` - 最大結果数（Discord制限は25）
    ///
    /// # Returns
    ///
    /// フィルタリングされたオートコンプリートオプションのリスト
    pub fn filter_by_partial(
        options: Vec<AutocompleteOption>,
        partial: &str,
        max_results: usize,
    ) -> Vec<AutocompleteOption> {
        let partial_lower = partial.to_lowercase();
        options
            .into_iter()
            .filter(|opt| {
                opt.name.to_lowercase().contains(&partial_lower)
                    || opt.value.to_lowercase().contains(&partial_lower)
            })
            .take(max_results)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_quest_items() {
        let items = vec![
            QuestAutocompleteItem {
                display_name: "天元たる六色の理".to_string(),
                quest_name: "天元たる六色の理".to_string(),
            },
            QuestAutocompleteItem {
                display_name: "ダーク・ラプチャー・ゼロ (ダクゼロ)".to_string(),
                quest_name: "ダーク・ラプチャー・ゼロ".to_string(),
            },
        ];

        let options = AutocompletePresenter::from_quest_items(items);

        assert_eq!(options.len(), 2);
        assert_eq!(options[0].name, "天元たる六色の理");
        assert_eq!(options[0].value, "天元たる六色の理");
        assert_eq!(options[1].name, "ダーク・ラプチャー・ゼロ (ダクゼロ)");
        assert_eq!(options[1].value, "ダーク・ラプチャー・ゼロ");
    }

    #[test]
    fn test_from_strings() {
        let items = vec!["火属性".to_string(), "水属性".to_string()];

        let options = AutocompletePresenter::from_strings(items);

        assert_eq!(options.len(), 2);
        assert_eq!(options[0].name, "火属性");
        assert_eq!(options[0].value, "火属性");
        assert_eq!(options[1].name, "水属性");
        assert_eq!(options[1].value, "水属性");
    }

    #[test]
    fn test_from_pairs() {
        let pairs = vec![
            ("Asia/Tokyo (JST)".to_string(), "Asia/Tokyo".to_string()),
            (
                "America/New_York (EST)".to_string(),
                "America/New_York".to_string(),
            ),
        ];

        let options = AutocompletePresenter::from_pairs(pairs);

        assert_eq!(options.len(), 2);
        assert_eq!(options[0].name, "Asia/Tokyo (JST)");
        assert_eq!(options[0].value, "Asia/Tokyo");
        assert_eq!(options[1].name, "America/New_York (EST)");
        assert_eq!(options[1].value, "America/New_York");
    }

    #[test]
    fn test_filter_by_partial() {
        let options = vec![
            AutocompleteOption::new("天元たる六色の理", "tengen"),
            AutocompleteOption::new("ダーク・ラプチャー・ゼロ", "darzero"),
            AutocompleteOption::new("天元クエスト", "tengen2"),
        ];

        // "天元"で検索
        let filtered = AutocompletePresenter::filter_by_partial(options.clone(), "天元", 25);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].name, "天元たる六色の理");
        assert_eq!(filtered[1].name, "天元クエスト");

        // "dar"で検索（値でも検索）
        let filtered = AutocompletePresenter::filter_by_partial(options.clone(), "dar", 25);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "ダーク・ラプチャー・ゼロ");

        // 最大結果数を1に制限
        let filtered = AutocompletePresenter::filter_by_partial(options, "天", 1);
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_filter_by_partial_case_insensitive() {
        let options = vec![
            AutocompleteOption::new("Test Quest", "test"),
            AutocompleteOption::new("Another Quest", "another"),
        ];

        // 大文字小文字を区別しない
        let filtered = AutocompletePresenter::filter_by_partial(options, "TEST", 25);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "Test Quest");
    }
}
