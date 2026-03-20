use super::{DateTimeParseOptions, ParsedDateTime, RelativeBase};
use crate::types::Result;
use chrono::Duration;

/// 文脈依存の相対時刻バリデーション
pub(super) fn validate_relative_for_context(
    parsed: &ParsedDateTime,
    options: &DateTimeParseOptions,
) -> Result<()> {
    let ParsedDateTime::Relative {
        days,
        hours,
        minutes,
    } = parsed
    else {
        return Ok(());
    };

    // 解散時刻は出発前のみ許可
    if matches!(options.relative_base, Some(RelativeBase::DateTime(_)))
        && (*days < 0 || *hours < 0 || *minutes < 0)
    {
        return Err("解散時刻の相対指定は「前」のみ指定できます"
            .to_string()
            .into());
    }

    if matches!(options.relative_base, Some(RelativeBase::DateTime(_))) {
        let total_minutes = *days as i64 * 24 * 60 + *hours as i64 * 60 + *minutes as i64;
        let max_minutes = dismissal_max_days() * 24 * 60;

        if total_minutes > max_minutes {
            return Err(format!(
                "解散時刻は出発時刻の{}日以内で指定してください",
                dismissal_max_days()
            )
            .into());
        }
    }

    Ok(())
}

/// 文脈依存の絶対日時バリデーション
pub(super) fn validate_absolute_for_context(
    parsed: &ParsedDateTime,
    options: &DateTimeParseOptions,
) -> Result<()> {
    let ParsedDateTime::Absolute(dt) = parsed else {
        return Ok(());
    };

    let Some(RelativeBase::DateTime(base)) = options.relative_base else {
        return Ok(());
    };

    if *dt > base {
        return Err("解散時刻は出発時刻より後に指定できません"
            .to_string()
            .into());
    }

    let diff = base - *dt;
    let max_days = dismissal_max_days();
    if diff > Duration::days(max_days) {
        return Err(format!("解散時刻は出発時刻の{}日以内で指定してください", max_days).into());
    }

    Ok(())
}

/// 解散時刻の許容日数を取得（未設定時は7日）
fn dismissal_max_days() -> i64 {
    std::env::var("DISMISSAL_MAX_DAYS")
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
        .filter(|v| *v >= 0)
        .unwrap_or(7)
}
