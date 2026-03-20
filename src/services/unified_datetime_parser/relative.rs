use super::ParsedDateTime;
use crate::services::number_normalizer::normalize_numbers;
use crate::types::Result;
use lazy_static::lazy_static;
use regex::Regex;

/// 相対時刻のパース
pub(super) fn parse_relative_time(input: &str) -> Result<Option<ParsedDateTime>> {
    lazy_static! {
        // 複数単位混在パターン: "1日2時間10分前", "1日1時間半前", "2h30m", "1 day 2 hours 10 minutes before"
        static ref RE_MULTI_UNIT: Regex = Regex::new(
            r"(?x)
            ^
            (?:(\d+)\s*(日|days?))?\s*                        # グループ1,2: 日（オプション）
            (?:(\d+)\s*(時間|hours?|h)\s*(半)?)?\s*          # グループ3,4,5: 時間と「半」（オプション）
            (?:(\d+)\s*(分|minutes?|mins?|m))?\s*             # グループ6,7: 分（オプション）
            $
            "
        )
        .expect("複数単位相対時刻Regexパターンが無効です");

        // 単一単位パターン（後方互換性）: "2時間前", "90m", "1day"
        static ref RE_SINGLE_UNIT: Regex = Regex::new(
            r"(?x)
            ^
            (\d+)\s*                    # 数値（スペース許可）
            (日|days?|時間|hours?|h|分|minutes?|mins?|m)      # 単位
            $
            "
        )
        .expect("単一単位相対時刻Regexパターンが無効です");

        // 「X時間半」パターン: "1時間半前", "2時間半"
        static ref RE_HOUR_HALF: Regex = Regex::new(
            r"(?x)
            ^
            (\d+)\s*                    # 数値
            (時間|hours?|h)\s*          # 時間単位
            半\s*                        # 「半」
            $
            "
        )
        .expect("時間半Regexパターンが無効です");
    }

    // 数字を正規化
    let normalized = normalize_numbers(input);
    let (body, direction) = extract_relative_direction(&normalized);

    // パターン1: 「X時間半」パターン（単独）
    if let Some(caps) = RE_HOUR_HALF.captures(body) {
        let hours = caps[1]
            .parse::<i32>()
            .map_err(|_| format!("数値のパースに失敗しました: {}", &caps[1]))?;

        return Ok(Some(ParsedDateTime::Relative {
            days: 0,
            hours: hours * direction,
            minutes: 30 * direction,
        }));
    }

    // パターン2: 複数単位混在パターン
    if let Some(caps) = RE_MULTI_UNIT.captures(body) {
        let days = caps
            .get(1)
            .and_then(|m| m.as_str().parse::<i32>().ok())
            .unwrap_or(0);

        let hours = caps
            .get(3)
            .and_then(|m| m.as_str().parse::<i32>().ok())
            .unwrap_or(0);

        let has_hour_half = caps.get(5).is_some();
        let hour_half_minutes = if has_hour_half { 30 } else { 0 };

        let minutes = caps
            .get(6)
            .and_then(|m| m.as_str().parse::<i32>().ok())
            .unwrap_or(0)
            + hour_half_minutes;

        if days > 0 || hours > 0 || minutes > 0 {
            return Ok(Some(ParsedDateTime::Relative {
                days: days * direction,
                hours: hours * direction,
                minutes: minutes * direction,
            }));
        }
    }

    // パターン3: 単一単位パターン（後方互換性）
    if let Some(caps) = RE_SINGLE_UNIT.captures(body) {
        let value = caps[1]
            .parse::<i32>()
            .map_err(|_| format!("数値のパースに失敗しました: {}", &caps[1]))?;
        let unit = &caps[2];

        let (days, hours, minutes) = match unit {
            "日" | "day" | "days" => (value, 0, 0),
            "時間" | "hour" | "hours" | "h" => (0, value, 0),
            "分" | "minute" | "minutes" | "min" | "mins" | "m" => (0, 0, value),
            _ => return Ok(None),
        };

        return Ok(Some(ParsedDateTime::Relative {
            days: days * direction,
            hours: hours * direction,
            minutes: minutes * direction,
        }));
    }

    Ok(None)
}

/// 相対時刻の方向を抽出
///
/// 戻り値:
/// - `&str`: 方向語を除いた本体
/// - `i32`: 方向（前/ago/before=1, 後/later/after=-1）
fn extract_relative_direction(input: &str) -> (&str, i32) {
    let trimmed = input.trim();

    if let Some(rest) = trimmed.strip_suffix('前') {
        return (rest.trim_end(), 1);
    }
    if let Some(rest) = trimmed.strip_suffix('後') {
        return (rest.trim_end(), -1);
    }

    let lowered = trimmed.to_ascii_lowercase();
    for (suffix, direction) in [
        (" before", 1),
        (" ago", 1),
        (" later", -1),
        (" after", -1),
        ("before", 1),
        ("ago", 1),
        ("later", -1),
        ("after", -1),
    ] {
        if lowered.ends_with(suffix) {
            let body_len = trimmed.len().saturating_sub(suffix.len());
            return (trimmed[..body_len].trim_end(), direction);
        }
    }

    // 方向指定なしは後方互換性のため「前」扱い
    (trimmed, 1)
}
