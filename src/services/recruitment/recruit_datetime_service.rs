use crate::repository::GuildSettingsRepository;
use crate::services::timezone_service::TimezoneService;
use crate::services::unified_datetime_parser::{
    DateTimeParseOptions, ParsedDateTime, parse_datetime,
};
use crate::types::{AppError, Result};
use chrono::{DateTime, Utc};
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

#[cfg(test)]
mod tests {
    use super::*;

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
