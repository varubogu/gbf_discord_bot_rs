use crate::types::{AppError, Result};
use std::collections::HashSet;

/// 曜日パースサービス
///
/// 曜日文字列の解析と整形を担当するサービス。
/// カンマ区切り、スペース区切り、連続パターン（「月火水」など）の各種フォーマットに対応。
pub struct DaysParserService;

impl DaysParserService {
    pub fn new() -> Self {
        Self
    }

    /// 曜日文字列をパース
    ///
    /// # 対応フォーマット:
    /// - 「毎日」「すべて」「all」→ 0（毎日を表す特殊な値）
    /// - カンマ区切り: 「月,水,金」→ [1, 3, 5]
    /// - スペース区切り: 「月 水 金」→ [1, 3, 5]
    /// - 連続パターン: 「月火水」→ [1, 2, 3]
    ///
    /// # 曜日番号:
    /// - 0: 毎日（特殊）
    /// - 1: 月曜日
    /// - 2: 火曜日
    /// - 3: 水曜日
    /// - 4: 木曜日
    /// - 5: 金曜日
    /// - 6: 土曜日
    /// - 7: 日曜日
    pub fn parse_days_input(&self, days_str: &str) -> Result<Vec<i32>> {
        // 無効なパターンを先にチェック
        if days_str == "曜日" {
            return Err(AppError::Business {
                message: "「曜日」は有効な曜日指定ではありません。「月」「火」などの曜日を指定してください。".to_string(),
            });
        }

        // 特殊パターンを先にチェック（「毎日」など）
        match days_str {
            "毎日" | "全て" | "すべて" | "everyday" | "all" => return Ok(vec![0]),
            _ => {}
        }

        // 区切り文字があるかチェック
        let has_delimiter =
            days_str.contains(',') || days_str.contains(' ') || days_str.contains('　');

        let result = if has_delimiter {
            // 区切り文字で分割して解析
            self.parse_days_with_delimiter(days_str)?
        } else {
            // 連続パターンとして解析（例: "月火水"）
            self.parse_continuous_pattern(days_str)?
        };

        Ok(result)
    }

    /// 区切り文字で分割された曜日をパース
    ///
    /// カンマ、半角スペース、全角スペースで分割して解析します。
    fn parse_days_with_delimiter(&self, days_str: &str) -> Result<Vec<i32>> {
        let mut result = Vec::new();

        // カンマ、半角スペース、全角スペースで分割
        for day in days_str.split([',', ' ', '　']) {
            let day = day.trim();

            // 空文字列はスキップ
            if day.is_empty() {
                continue;
            }

            let day_num = self.parse_single_day(day)?;
            result.push(day_num);
        }

        Ok(result)
    }

    /// 連続パターンの曜日をパース（例: "月火水" → [1, 2, 3]）
    ///
    /// 重複する曜日は自動的に除去され、ソートされた結果を返します。
    fn parse_continuous_pattern(&self, days_str: &str) -> Result<Vec<i32>> {
        let mut result_set = HashSet::new();

        for ch in days_str.chars() {
            match ch {
                '月' => {
                    result_set.insert(1);
                }
                '火' => {
                    result_set.insert(2);
                }
                '水' => {
                    result_set.insert(3);
                }
                '木' => {
                    result_set.insert(4);
                }
                '金' => {
                    result_set.insert(5);
                }
                '土' => {
                    result_set.insert(6);
                }
                '日' => {
                    result_set.insert(7);
                }
                '曜' => continue, // 「曜」はスキップ（"月曜火曜" のような入力に対応）
                _ => {
                    return Err(AppError::Business {
                        message: format!(
                            "無効な文字が含まれています: '{ch}'（使用可能: 月火水木金土日）"
                        ),
                    });
                }
            };
        }

        if result_set.is_empty() {
            return Err(AppError::Business {
                message: "有効な曜日が指定されていません".to_string(),
            });
        }

        // ソートして返す
        let mut result: Vec<i32> = result_set.into_iter().collect();
        result.sort();
        Ok(result)
    }

    /// 単一の曜日文字列を曜日番号に変換
    fn parse_single_day(&self, day: &str) -> Result<i32> {
        match day {
            "毎日" | "全て" | "すべて" | "everyday" | "all" => Ok(0),
            "月" | "月曜" | "月曜日" | "mon" | "monday" => Ok(1),
            "火" | "火曜" | "火曜日" | "tue" | "tuesday" => Ok(2),
            "水" | "水曜" | "水曜日" | "wed" | "wednesday" => Ok(3),
            "木" | "木曜" | "木曜日" | "thu" | "thursday" => Ok(4),
            "金" | "金曜" | "金曜日" | "fri" | "friday" => Ok(5),
            "土" | "土曜" | "土曜日" | "sat" | "saturday" => Ok(6),
            "日" | "日曜" | "日曜日" | "sun" | "sunday" => Ok(7),
            _ => Err(AppError::Business {
                message: format!("無効な曜日です: {day}"),
            }),
        }
    }

    /// 曜日番号リストを表示用文字列に整形
    ///
    /// 例: [1, 3, 5] → "月, 水, 金"
    pub fn format_days(&self, days: &[i32]) -> String {
        let day_names: Vec<String> = days
            .iter()
            .map(|&d| match d {
                0 => "毎日".to_string(),
                1 => "月".to_string(),
                2 => "火".to_string(),
                3 => "水".to_string(),
                4 => "木".to_string(),
                5 => "金".to_string(),
                6 => "土".to_string(),
                7 => "日".to_string(),
                _ => format!("不明({d})"),
            })
            .collect();

        day_names.join(", ")
    }
}

impl Default for DaysParserService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_days_comma_separated() {
        let service = DaysParserService::new();
        let result = service.parse_days_input("月,水,金").unwrap();
        assert_eq!(result, vec![1, 3, 5]);
    }

    #[test]
    fn test_parse_days_space_separated() {
        let service = DaysParserService::new();
        let result = service.parse_days_input("月 水 金").unwrap();
        assert_eq!(result, vec![1, 3, 5]);
    }

    #[test]
    fn test_parse_days_fullwidth_space_separated() {
        let service = DaysParserService::new();
        let result = service.parse_days_input("月　水　金").unwrap();
        assert_eq!(result, vec![1, 3, 5]);
    }

    #[test]
    fn test_parse_days_mixed_delimiters() {
        let service = DaysParserService::new();
        let result = service.parse_days_input("月,水　金").unwrap();
        assert_eq!(result, vec![1, 3, 5]);
    }

    #[test]
    fn test_parse_days_with_extra_spaces() {
        let service = DaysParserService::new();
        let result = service.parse_days_input("月,  水,  金").unwrap();
        assert_eq!(result, vec![1, 3, 5]);
    }

    #[test]
    fn test_parse_days_everyday() {
        let service = DaysParserService::new();
        let result = service.parse_days_input("毎日").unwrap();
        assert_eq!(result, vec![0]);
    }

    #[test]
    fn test_parse_days_invalid() {
        let service = DaysParserService::new();
        let result = service.parse_days_input("月,無効,金");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_days_continuous_pattern() {
        let service = DaysParserService::new();
        let result = service.parse_days_input("月火水").unwrap();
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn test_parse_days_continuous_pattern_weekend() {
        let service = DaysParserService::new();
        let result = service.parse_days_input("金土日").unwrap();
        assert_eq!(result, vec![5, 6, 7]);
    }

    #[test]
    fn test_parse_days_continuous_with_youbi() {
        let service = DaysParserService::new();
        // "月曜火曜" のような入力（「曜」はスキップされる）
        let result = service.parse_days_input("月曜火曜").unwrap();
        assert_eq!(result, vec![1, 2]);
    }

    #[test]
    fn test_parse_days_continuous_duplicates() {
        let service = DaysParserService::new();
        // 重複は自動的に除去される
        let result = service.parse_days_input("月月火水").unwrap();
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn test_parse_days_youbi_only() {
        let service = DaysParserService::new();
        // "曜日" は有効な曜日指定ではないためエラー
        let result = service.parse_days_input("曜日");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_days_sunday_only() {
        let service = DaysParserService::new();
        // "日" のみは日曜日として扱う
        let result = service.parse_days_input("日").unwrap();
        assert_eq!(result, vec![7]);
    }

    #[test]
    fn test_parse_days_continuous_invalid_char() {
        let service = DaysParserService::new();
        // 無効な文字が含まれている場合はエラー
        let result = service.parse_days_input("月火ABC");
        assert!(result.is_err());
    }

    #[test]
    fn test_format_days() {
        let service = DaysParserService::new();
        let formatted = service.format_days(&[1, 3, 5]);
        assert_eq!(formatted, "月, 水, 金");
    }

    #[test]
    fn test_format_days_everyday() {
        let service = DaysParserService::new();
        let formatted = service.format_days(&[0]);
        assert_eq!(formatted, "毎日");
    }

    #[test]
    fn test_format_days_weekend() {
        let service = DaysParserService::new();
        let formatted = service.format_days(&[6, 7]);
        assert_eq!(formatted, "土, 日");
    }
}
