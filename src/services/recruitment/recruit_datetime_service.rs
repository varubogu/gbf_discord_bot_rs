use crate::repository::GuildSettingsRepository;
use crate::services::timezone_service::TimezoneService;
use crate::services::unified_datetime_parser::{
    DateTimeParseOptions, ParsedDateTime, parse_datetime,
};
use crate::types::{AppError, PostponeDepartureResult, Result};
use chrono::{DateTime, Duration, Utc};
use chrono_tz::Tz;
use sea_orm::{DatabaseConnection, DatabaseTransaction};

/// 募集系コマンドの日時パースを集約するサービス
pub struct RecruitDateTimeService<R>
where
    R: GuildSettingsRepository,
{
    timezone_service: TimezoneService<R>,
}

impl<R> RecruitDateTimeService<R>
where
    R: GuildSettingsRepository,
{
    pub fn new(guild_settings_repo: R) -> Self {
        Self {
            timezone_service: TimezoneService::new(guild_settings_repo),
        }
    }

    /// クエスト出発日時をギルド設定タイムゾーンで解釈してUTCへ変換する
    ///
    /// 絶対日時のみ受け付ける。
    pub async fn parse_quest_departure(
        &self,
        db: &DatabaseConnection,
        guild_id: i64,
        input: &str,
    ) -> Result<DateTime<Utc>> {
        let timezone = self
            .timezone_service
            .get_guild_timezone(db, guild_id)
            .await?;
        parse_quest_departure_datetime(input, timezone)
    }

    /// クエスト出発日時をギルド設定タイムゾーンで解釈してUTCへ変換する（トランザクション版）
    ///
    /// 絶対日時のみ受け付ける。
    pub async fn parse_quest_departure_with_txn(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        input: &str,
    ) -> Result<DateTime<Utc>> {
        let timezone = self
            .timezone_service
            .get_guild_timezone_with_txn(txn, guild_id)
            .await?;
        parse_quest_departure_datetime(input, timezone)
    }
}

/// クエスト出発日時をパースする（絶対日時のみ）
pub fn parse_quest_departure_datetime(input: &str, timezone: Tz) -> Result<DateTime<Utc>> {
    let options = DateTimeParseOptions::for_quest_departure(timezone);
    let results = parse_datetime(input, &options)?;

    match results.first() {
        Some(ParsedDateTime::Absolute(dt)) => Ok(*dt),
        _ => Err(AppError::Business {
            message: "クエスト出発日時は絶対日時で指定してください".to_string(),
        }),
    }
}

/// クエスト出発日時を指定分だけ後ろへずらす
///
/// 既に出発時刻を過ぎている募集は後ろ倒しできない（後ろ倒ししても過去のままとなり、
/// 出発通知が作成されず投稿削除予定だけが過去時刻で再作成されるため）。
///
/// # 引数
/// * `current_departure` - 現在の出発日時
/// * `minutes` - 後ろへずらす分数
/// * `now` - 判定基準となる現在時刻
pub fn postpone_quest_departure(
    current_departure: DateTime<Utc>,
    minutes: i64,
    now: DateTime<Utc>,
) -> Result<PostponeDepartureResult> {
    if current_departure <= now {
        return Ok(PostponeDepartureResult::EventDatePassed);
    }

    let postponed = current_departure
        .checked_add_signed(Duration::minutes(minutes))
        .ok_or_else(|| AppError::Validation {
            field: "出発日時".to_string(),
        })?;

    Ok(PostponeDepartureResult::Postponed(postponed))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// テスト用にUTCの日時を組み立てる
    fn utc(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> DateTime<Utc> {
        use chrono::TimeZone;
        Utc.with_ymd_and_hms(year, month, day, hour, minute, 0)
            .single()
            .expect("テスト用日時は一意に定まるべき")
    }

    #[test]
    fn postpone_quest_departure_未出発なら指定分だけ後ろへずらす() {
        // Arrange
        let now = utc(2026, 1, 1, 21, 0);
        let departure = utc(2026, 1, 1, 22, 0);

        // Act
        let result = postpone_quest_departure(departure, 30, now);

        // Assert
        assert_eq!(
            result.expect("後ろ倒しは成功するべき"),
            PostponeDepartureResult::Postponed(utc(2026, 1, 1, 22, 30))
        );
    }

    #[test]
    fn postpone_quest_departure_出発時刻を過ぎていたら後ろ倒ししない() {
        // Arrange
        let now = utc(2026, 1, 1, 23, 0);
        let departure = utc(2026, 1, 1, 22, 0);

        // Act
        let result = postpone_quest_departure(departure, 30, now);

        // Assert
        assert_eq!(
            result.expect("判定自体は成功するべき"),
            PostponeDepartureResult::EventDatePassed
        );
    }

    #[test]
    fn postpone_quest_departure_出発時刻ちょうどは後ろ倒ししない() {
        // Arrange
        let now = utc(2026, 1, 1, 22, 0);
        let departure = utc(2026, 1, 1, 22, 0);

        // Act
        let result = postpone_quest_departure(departure, 30, now);

        // Assert
        assert_eq!(
            result.expect("判定自体は成功するべき"),
            PostponeDepartureResult::EventDatePassed
        );
    }

    #[test]
    fn parse_quest_departure_datetime_絶対日時を受け付ける() {
        let parsed = parse_quest_departure_datetime("12/25 22:30", chrono_tz::Asia::Tokyo);
        assert!(parsed.is_ok());
    }

    #[test]
    fn parse_quest_departure_datetime_相対時刻を拒否する() {
        let parsed = parse_quest_departure_datetime("1時間後", chrono_tz::Asia::Tokyo);
        assert!(parsed.is_err());
    }
}
