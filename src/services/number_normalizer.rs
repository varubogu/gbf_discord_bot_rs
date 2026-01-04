/// 全角数字・漢数字を半角数字に正規化する
///
/// # 対応形式
/// - 全角数字: ０１２３４５６７８９ → 0123456789
/// - 漢数字（位取りなし）: 一二三四五六七八九〇 → 123456789
/// - 漢数字（十の位）: 十、二十、三十、二十八... → 10, 20, 30, 28...
/// - 混在: １日二時間３０分前 → 1日2時間30分前
pub fn normalize_numbers(input: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];
        match ch {
            // 全角数字を半角に変換
            '０' => result.push('0'),
            '１' => result.push('1'),
            '２' => result.push('2'),
            '３' => result.push('3'),
            '４' => result.push('4'),
            '５' => result.push('5'),
            '６' => result.push('6'),
            '７' => result.push('7'),
            '８' => result.push('8'),
            '９' => result.push('9'),

            // 漢数字
            '〇' => result.push('0'),
            '一' | '二' | '三' | '四' | '五' | '六' | '七' | '八' | '九' => {
                let digit = match ch {
                    '一' => '1',
                    '二' => '2',
                    '三' => '3',
                    '四' => '4',
                    '五' => '5',
                    '六' => '6',
                    '七' => '7',
                    '八' => '8',
                    '九' => '9',
                    _ => unreachable!(),
                };

                // 次が「十」かチェック
                if i + 1 < chars.len() && chars[i + 1] == '十' {
                    // さらにその次が一桁の漢数字かチェック（例: 二十八 → 28）
                    if i + 2 < chars.len() {
                        match chars[i + 2] {
                            '一' | '二' | '三' | '四' | '五' | '六' | '七' | '八' | '九' =>
                            {
                                let ones_digit = match chars[i + 2] {
                                    '一' => '1',
                                    '二' => '2',
                                    '三' => '3',
                                    '四' => '4',
                                    '五' => '5',
                                    '六' => '6',
                                    '七' => '7',
                                    '八' => '8',
                                    '九' => '9',
                                    _ => unreachable!(),
                                };
                                // 「X十Y」→「XY」（例: 二十八 → 28）
                                result.push(digit);
                                result.push(ones_digit);
                                i += 2; // '十'と一の位をスキップ
                            }
                            _ => {
                                // 「X十」→「X0」（例: 二十 → 20）
                                result.push(digit);
                                result.push('0');
                                i += 1; // '十'をスキップ
                            }
                        }
                    } else {
                        // 「X十」→「X0」（例: 二十 → 20）
                        result.push(digit);
                        result.push('0');
                        i += 1; // '十'をスキップ
                    }
                } else {
                    result.push(digit);
                }
            }
            '十' => {
                // 次が一桁の漢数字かチェック（例: 十八 → 18）
                if i + 1 < chars.len() {
                    match chars[i + 1] {
                        '一' | '二' | '三' | '四' | '五' | '六' | '七' | '八' | '九' => {
                            let ones_digit = match chars[i + 1] {
                                '一' => '1',
                                '二' => '2',
                                '三' => '3',
                                '四' => '4',
                                '五' => '5',
                                '六' => '6',
                                '七' => '7',
                                '八' => '8',
                                '九' => '9',
                                _ => unreachable!(),
                            };
                            // 「十Y」→「1Y」（例: 十八 → 18）
                            result.push('1');
                            result.push(ones_digit);
                            i += 1; // 一の位をスキップ
                        }
                        _ => {
                            // 単独の「十」→「10」
                            result.push_str("10");
                        }
                    }
                } else {
                    // 単独の「十」→「10」
                    result.push_str("10");
                }
            }

            // その他はそのまま
            _ => result.push(ch),
        }
        i += 1;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_fullwidth_numbers() {
        assert_eq!(normalize_numbers("１２３"), "123");
        assert_eq!(normalize_numbers("０"), "0");
        assert_eq!(normalize_numbers("９"), "9");
    }

    #[test]
    fn test_normalize_kanji_numbers() {
        assert_eq!(normalize_numbers("一二三"), "123");
        assert_eq!(normalize_numbers("〇"), "0");
        assert_eq!(normalize_numbers("十"), "10");
        assert_eq!(normalize_numbers("二十"), "20");
        assert_eq!(normalize_numbers("三十"), "30");
        assert_eq!(normalize_numbers("二十八"), "28");
        assert_eq!(normalize_numbers("十八"), "18");
        assert_eq!(normalize_numbers("三十一"), "31");
    }

    #[test]
    fn test_normalize_mixed() {
        assert_eq!(normalize_numbers("１日二時間３０分前"), "1日2時間30分前");
        assert_eq!(normalize_numbers("一日二十分前"), "1日20分前");
        assert_eq!(normalize_numbers("２８ １０００"), "28 1000");
    }

    #[test]
    fn test_normalize_datetime_patterns() {
        // 日時パターン
        assert_eq!(normalize_numbers("１２/１１ １４:００"), "12/11 14:00");
        assert_eq!(normalize_numbers("二十八 １０００"), "28 1000");
        assert_eq!(normalize_numbers("十 1230"), "10 1230");
    }
}
