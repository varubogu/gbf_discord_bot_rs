/// 統一日時パーサー
///
/// ビットフラグベースの柔軟な日時解析システム。
/// 既存の複数のパーサー（datetime_parser, TimeParserService, DismissalTimeParserService）を統合。
use crate::types::Result;
use chrono::{DateTime, NaiveTime, Utc};
use chrono_tz::Tz;

mod absolute;
mod parser;
mod relative;
mod validation;

#[cfg(test)]
mod tests;

/// 日時解析パターンフラグ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateTimeParseFlags {
    bits: u32,
}

impl DateTimeParseFlags {
    /// 完全日時: "2025/11/15 21:00", "2025-11-15 21:00"
    pub const FULL_DATETIME: Self = Self { bits: 0b00000001 };

    /// 年なし日時: "12/11 14:00", "12-11 14:00"
    pub const DATETIME_NO_YEAR: Self = Self { bits: 0b00000010 };

    /// 日付のみ: "11/15", "11-15"
    pub const DATE_ONLY: Self = Self { bits: 0b00000100 };

    /// 時刻のみ: "21:00", "21時"
    pub const TIME_ONLY: Self = Self { bits: 0b00001000 };

    /// 日本語日時: "1月2日3時4分", "午後9時半"
    pub const JAPANESE_DATETIME: Self = Self { bits: 0b00010000 };

    /// 数字パターン: "1230", "10111230", "30 1230"
    pub const NUMERIC_PATTERNS: Self = Self { bits: 0b00100000 };

    /// 相対時刻: "2時間前", "1day", "90分前"
    pub const RELATIVE_TIME: Self = Self { bits: 0b01000000 };

    /// 絶対日時系のフラグ集合
    pub const ABSOLUTE: Self = Self {
        bits: Self::FULL_DATETIME.bits
            | Self::DATETIME_NO_YEAR.bits
            | Self::DATE_ONLY.bits
            | Self::TIME_ONLY.bits
            | Self::JAPANESE_DATETIME.bits
            | Self::NUMERIC_PATTERNS.bits,
    };

    /// すべてのパターンを許可
    pub const ALL: Self = Self {
        bits: Self::ABSOLUTE.bits | Self::RELATIVE_TIME.bits,
    };

    /// 何も許可しない（空）
    pub const NONE: Self = Self { bits: 0 };

    /// フラグの結合
    pub const fn union(self, other: Self) -> Self {
        Self {
            bits: self.bits | other.bits,
        }
    }

    /// フラグの除外
    pub const fn difference(self, other: Self) -> Self {
        Self {
            bits: self.bits & !other.bits,
        }
    }

    /// フラグが含まれているか
    pub const fn contains(self, other: Self) -> bool {
        (self.bits & other.bits) == other.bits
    }

    /// フラグの交差があるか
    pub const fn intersects(self, other: Self) -> bool {
        (self.bits & other.bits) != 0
    }

    /// フラグが空か
    pub const fn is_empty(self) -> bool {
        self.bits == 0
    }
}

/// ビット演算のための実装
impl std::ops::BitOr for DateTimeParseFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(rhs)
    }
}

impl std::ops::BitAnd for DateTimeParseFlags {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            bits: self.bits & rhs.bits,
        }
    }
}

impl std::ops::Not for DateTimeParseFlags {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self { bits: !self.bits }
    }
}

/// 相対時刻の基準
#[derive(Debug, Clone)]
pub enum RelativeBase {
    /// DateTime基準（解散時刻などで使用）
    DateTime(DateTime<Utc>),

    /// NaiveTime基準（定期募集開始時刻などで使用）
    Time(NaiveTime),
}

/// 日時解析オプション
#[derive(Debug, Clone)]
pub struct DateTimeParseOptions {
    /// 許可するパターンフラグ
    pub flags: DateTimeParseFlags,

    /// タイムゾーン
    pub timezone: Tz,

    /// 相対時刻の基準（RELATIVE_TIME有効時に必要）
    pub relative_base: Option<RelativeBase>,

    /// デフォルト時刻（DATE_ONLYで日付のみの場合に使用）
    pub default_time: Option<NaiveTime>,

    /// 複数時刻の許可（カンマ区切り）
    pub allow_multiple: bool,

    /// 最大個数（allow_multiple=true時）
    pub max_count: usize,
}

impl DateTimeParseOptions {
    /// クエスト出発日時用（絶対日時のみ、多様なパターン）
    pub fn for_quest_departure(timezone: Tz) -> Self {
        let default_time = NaiveTime::from_hms_opt(21, 0, 0).unwrap_or(NaiveTime::MIN);
        Self {
            flags: DateTimeParseFlags::ALL.difference(DateTimeParseFlags::RELATIVE_TIME),
            timezone,
            relative_base: None,
            default_time: Some(default_time),
            allow_multiple: false,
            max_count: 1,
        }
    }

    /// 解散時刻用（相対・絶対両方、複数可、最大3つ）
    pub fn for_dismissal_time(timezone: Tz, base_datetime: DateTime<Utc>) -> Self {
        Self {
            flags: DateTimeParseFlags::ALL,
            timezone,
            relative_base: Some(RelativeBase::DateTime(base_datetime)),
            default_time: None,
            allow_multiple: true,
            max_count: 3,
        }
    }

    /// 定期募集開始時刻用（時刻のみ + 相対時刻）
    pub fn for_schedule_start_time(timezone: Tz, base_time: NaiveTime) -> Self {
        Self {
            flags: DateTimeParseFlags::TIME_ONLY
                | DateTimeParseFlags::JAPANESE_DATETIME
                | DateTimeParseFlags::NUMERIC_PATTERNS
                | DateTimeParseFlags::RELATIVE_TIME,
            timezone,
            relative_base: Some(RelativeBase::Time(base_time)),
            default_time: None,
            allow_multiple: false,
            max_count: 1,
        }
    }

    /// HH:MM厳格モード（既存TimeParserService互換）
    pub fn strict_hhmm_only(timezone: Tz) -> Self {
        Self {
            flags: DateTimeParseFlags::NONE,
            timezone,
            relative_base: None,
            default_time: None,
            allow_multiple: false,
            max_count: 1,
        }
    }
}

/// 解析結果
#[derive(Debug, Clone, PartialEq)]
pub enum ParsedDateTime {
    /// 絶対日時
    Absolute(DateTime<Utc>),

    /// 相対時刻（基準時刻からのオフセット）
    Relative { days: i32, hours: i32, minutes: i32 },

    /// NaiveTime（定期募集開始時刻など）
    Time(NaiveTime),
}

/// パース済み解散時刻（後方互換性のため）
#[derive(Debug, Clone, PartialEq)]
pub enum ParsedDismissalTime {
    /// 絶対日時
    Absolute {
        input_value: String,
        datetime: DateTime<Utc>,
    },
    /// 相対時刻
    Relative {
        input_value: String,
        days: i32,
        hours: i32,
        minutes: i32,
    },
}

/// 統一日時パーサー
///
/// # 引数
/// - `input`: 解析する文字列
/// - `options`: 解析オプション
///
/// # 戻り値
/// パース済み日時のベクタ（allow_multiple=falseの場合は要素数1）
///
/// # エラー
/// - パースに失敗した場合
/// - 最大個数を超えた場合
pub fn parse_datetime(input: &str, options: &DateTimeParseOptions) -> Result<Vec<ParsedDateTime>> {
    let trimmed = input.trim();

    if options.allow_multiple {
        let parts: Vec<&str> = trimmed
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        if parts.len() > options.max_count {
            return Err(format!(
                "最大{}つまで指定できます（指定された数: {}）",
                options.max_count,
                parts.len()
            )
            .into());
        }

        let mut results = Vec::with_capacity(parts.len());
        for part in parts {
            results.push(parser::parse_single(part, options)?);
        }
        return Ok(results);
    }

    Ok(vec![parser::parse_single(trimmed, options)?])
}
