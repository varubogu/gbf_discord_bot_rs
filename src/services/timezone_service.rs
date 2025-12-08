use crate::repository::database::guild_timezone_repository::GuildTimezoneRepository;
use crate::types::{AppError, Result};
use chrono_tz::Tz;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tracing::{debug, error, info};

/// タイムゾーン取得・設定サービス
pub struct TimezoneService {
    repository: Arc<GuildTimezoneRepository>,
}

impl TimezoneService {
    pub fn new(repository: Arc<GuildTimezoneRepository>) -> Self {
        Self { repository }
    }

    /// ギルドのタイムゾーンを取得
    /// 未設定の場合はデフォルト（Asia/Tokyo）を返す
    pub async fn get_guild_timezone(&self, db: &DatabaseConnection, guild_id: i64) -> Result<Tz> {
        debug!(guild_id = guild_id, "ギルドのタイムゾーンを取得します");

        match self.repository.find_by_guild_id(db, guild_id).await? {
            Some(settings) => {
                // タイムゾーン文字列をTz型に変換
                let tz = settings.timezone.parse::<Tz>().map_err(|e| {
                    error!(
                        error = %e,
                        timezone = settings.timezone,
                        "無効なタイムゾーン名がDBに保存されています"
                    );
                    AppError::Validation {
                        field: format!("timezone: {}", settings.timezone),
                    }
                })?;

                info!(
                    guild_id = guild_id,
                    timezone = %tz,
                    "ギルドのタイムゾーンを取得しました"
                );

                Ok(tz)
            }
            None => {
                // 未設定の場合はデフォルト（Asia/Tokyo）を返す
                debug!(
                    guild_id = guild_id,
                    "タイムゾーン未設定のため、デフォルト（Asia/Tokyo）を使用します"
                );
                Ok(chrono_tz::Asia::Tokyo)
            }
        }
    }

    /// タイムゾーン名のバリデーション
    pub fn validate_timezone(timezone_str: &str) -> Result<Tz> {
        timezone_str.parse::<Tz>().map_err(|_| {
            AppError::Validation {
                field: format!(
                    "timezone: {}（IANAタイムゾーン名を指定してください。例: Asia/Tokyo, America/New_York）",
                    timezone_str
                ),
            }
        })
    }
}
