use super::absolute::{self, AbsoluteCategory};
use super::relative;
use super::validation;
use super::{DateTimeParseFlags, DateTimeParseOptions, ParsedDateTime, RelativeBase};
use crate::types::Result;
use chrono::{Duration, NaiveDateTime, TimeZone, Utc};

/// 単一の日時文字列をパース
pub(super) fn parse_single(input: &str, options: &DateTimeParseOptions) -> Result<ParsedDateTime> {
    // HH:MM厳格モード
    if options.flags.is_empty() {
        return parse_strict_hhmm(input);
    }

    // 絶対日時を先に試行（"20時" などの曖昧入力を時刻として優先解釈）
    if let Some(candidate) = absolute::parse_absolute_candidate(input, options)?
        && candidate.is_allowed_by(options.flags)
    {
        let absolute = convert_absolute_candidate(candidate, options)?;
        validation::validate_absolute_for_context(&absolute, options)?;
        return Ok(absolute);
    }

    // 相対時刻を試行（絶対日時で解釈できない場合）
    if options.flags.contains(DateTimeParseFlags::RELATIVE_TIME)
        && let Some(relative) = relative::parse_relative_time(input)?
    {
        validation::validate_relative_for_context(&relative, options)?;
        return Ok(relative);
    }

    Err(format!("日時のパースに失敗しました: {input}").into())
}

/// 絶対日時の候補を呼び出し側の文脈に合わせて変換する
fn convert_absolute_candidate(
    candidate: absolute::AbsoluteParseCandidate,
    options: &DateTimeParseOptions,
) -> Result<ParsedDateTime> {
    if candidate.category == AbsoluteCategory::TimeOnly {
        let local_time = candidate.datetime.with_timezone(&options.timezone).time();

        // 定期募集開始時刻のように「時刻値」として返したい場合
        if matches!(options.relative_base, Some(RelativeBase::Time(_))) {
            return Ok(ParsedDateTime::Time(local_time));
        }

        // 解散時刻のように「出発日時基準」で当日/前日を決めたい場合
        if let Some(RelativeBase::DateTime(base_datetime)) = options.relative_base {
            let base_local = base_datetime.with_timezone(&options.timezone);
            let mut dismissal_local = options
                .timezone
                .from_local_datetime(&NaiveDateTime::new(base_local.date_naive(), local_time))
                .single()
                .ok_or_else(|| "曖昧な時刻またはサマータイム切り替え時刻です".to_string())?;

            // 出発時刻と同時刻以上なら前日に寄せる
            if dismissal_local >= base_local {
                dismissal_local -= Duration::days(1);
            }

            return Ok(ParsedDateTime::Absolute(
                dismissal_local.with_timezone(&Utc),
            ));
        }
    }

    Ok(ParsedDateTime::Absolute(candidate.datetime))
}

/// HH:MM厳格モードのパース
fn parse_strict_hhmm(input: &str) -> Result<ParsedDateTime> {
    let parts: Vec<&str> = input.split(':').collect();

    if parts.len() != 2 {
        return Err(format!("無効な時刻形式です: {input}（HH:MM形式で指定してください）").into());
    }

    let hour = parts[0]
        .parse::<u32>()
        .map_err(|_| format!("無効な時刻です: {input}"))?;

    let minute = parts[1]
        .parse::<u32>()
        .map_err(|_| format!("無効な時刻です: {input}"))?;

    let naive_time = chrono::NaiveTime::from_hms_opt(hour, minute, 0)
        .ok_or_else(|| format!("無効な時刻です: {input}"))?;

    Ok(ParsedDateTime::Time(naive_time))
}
