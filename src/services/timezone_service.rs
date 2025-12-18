use crate::repository::database::guild_timezone_repository::GuildTimezoneRepository;
use crate::types::{AppError, Result};
use chrono::{Offset, TimeZone, Utc};
use chrono_tz::Tz;
use lazy_static::lazy_static;
use poise::serenity_prelude::AutocompleteChoice;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tracing::{debug, error, info};

/// タイムゾーンのオートコンプリート用データ
#[derive(Clone)]
struct TimezoneChoiceData {
    display_name: String,
    value: String,
}

/// 主要なタイムゾーンのリスト（名前、IANAタイムゾーン名、説明）
/// 表示名は「地域名 (UTC+9:00)」形式
const COMMON_TIMEZONES: &[(&str, &str)] = &[
    // アジア
    ("Asia/Tokyo", "日本標準時 (JST)"),
    ("Asia/Seoul", "韓国標準時 (KST)"),
    ("Asia/Shanghai", "中国標準時 (CST)"),
    ("Asia/Hong_Kong", "香港時間 (HKT)"),
    ("Asia/Taipei", "台湾標準時 (CST)"),
    ("Asia/Singapore", "シンガポール標準時 (SGT)"),
    ("Asia/Bangkok", "インドシナ時間 (ICT)"),
    ("Asia/Jakarta", "西部インドネシア時間 (WIB)"),
    ("Asia/Manila", "フィリピン標準時 (PST)"),
    ("Asia/Kolkata", "インド標準時 (IST)"),
    ("Asia/Dubai", "湾岸標準時 (GST)"),

    // オセアニア
    ("Australia/Sydney", "オーストラリア東部標準時 (AEST)"),
    ("Australia/Melbourne", "オーストラリア東部標準時 (AEST)"),
    ("Australia/Perth", "オーストラリア西部標準時 (AWST)"),
    ("Pacific/Auckland", "ニュージーランド標準時 (NZST)"),

    // 北米
    ("America/New_York", "米国東部標準時 (EST)"),
    ("America/Chicago", "米国中部標準時 (CST)"),
    ("America/Denver", "米国山岳部標準時 (MST)"),
    ("America/Los_Angeles", "米国太平洋標準時 (PST)"),
    ("America/Anchorage", "アラスカ標準時 (AKST)"),
    ("America/Toronto", "カナダ東部標準時 (EST)"),
    ("America/Vancouver", "カナダ太平洋標準時 (PST)"),

    // 中南米
    ("America/Mexico_City", "メキシコ中部標準時 (CST)"),
    ("America/Sao_Paulo", "ブラジリア時間 (BRT)"),
    ("America/Buenos_Aires", "アルゼンチン時間 (ART)"),

    // ヨーロッパ
    ("Europe/London", "グリニッジ標準時 (GMT)"),
    ("Europe/Paris", "中央ヨーロッパ標準時 (CET)"),
    ("Europe/Berlin", "中央ヨーロッパ標準時 (CET)"),
    ("Europe/Rome", "中央ヨーロッパ標準時 (CET)"),
    ("Europe/Madrid", "中央ヨーロッパ標準時 (CET)"),
    ("Europe/Moscow", "モスクワ標準時 (MSK)"),

    // アフリカ
    ("Africa/Cairo", "東ヨーロッパ標準時 (EET)"),
    ("Africa/Johannesburg", "南アフリカ標準時 (SAST)"),

    // UTC
    ("UTC", "協定世界時 (UTC)"),
];

// オートコンプリート用のタイムゾーン候補リスト（キャッシュ）
// プログラム起動時に計算され、以後は静的に保持される
lazy_static! {
    static ref TIMEZONE_CHOICE_DATA: Vec<TimezoneChoiceData> = {
        let now = Utc::now();

        COMMON_TIMEZONES
            .iter()
            .map(|(tz_name, description)| {
                // UTCオフセットを計算
                let offset_str = if let Ok(tz) = tz_name.parse::<Tz>() {
                    let offset = tz.offset_from_utc_datetime(&now.naive_utc());
                    let total_seconds = offset.fix().local_minus_utc();
                    let hours = total_seconds / 3600;
                    let minutes = (total_seconds % 3600) / 60;

                    if minutes == 0 {
                        format!("UTC{:+}", hours)
                    } else {
                        format!("UTC{:+}:{:02}", hours, minutes.abs())
                    }
                } else {
                    "UTC+0".to_string()
                };

                // 表示名: "Asia/Tokyo - 日本標準時 (JST) [UTC+9]"
                let display_name = format!("{} - {} [{}]", tz_name, description, offset_str);

                TimezoneChoiceData {
                    display_name,
                    value: tz_name.to_string(),
                }
            })
            .collect()
    };
}

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

    /// ギルドのタイムゾーンを設定（upsert）
    pub async fn set_guild_timezone(
        &self,
        txn: &sea_orm::DatabaseTransaction,
        guild_id: i64,
        timezone_name: &str,
    ) -> Result<()> {
        self.repository
            .upsert_with_txn(txn, guild_id, timezone_name)
            .await?;

        info!(
            guild_id = guild_id,
            timezone = timezone_name,
            "ギルドのタイムゾーンを設定しました"
        );

        Ok(())
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

    /// オートコンプリート用のタイムゾーンリストを取得
    /// 部分文字列でフィルタリングし、UTC+9:00形式のオフセット付きで表示
    /// キャッシュを使用してパフォーマンスを最適化
    pub fn get_timezones_for_autocomplete(partial: &str) -> Vec<AutocompleteChoice> {
        // 部分文字列でフィルタリング
        let partial_lower = partial.to_lowercase();
        TIMEZONE_CHOICE_DATA
            .iter()
            .filter(|data| {
                // 表示名または値に部分文字列が含まれる
                data.display_name.to_lowercase().contains(&partial_lower)
                    || data.value.to_lowercase().contains(&partial_lower)
            })
            .take(25) // Discordの制限
            .map(|data| AutocompleteChoice::new(data.display_name.clone(), data.value.clone()))
            .collect()
    }

    /// タイムゾーンキャッシュを初期化する
    /// プログラム起動時に呼び出すことで、事前に計算を完了させる
    pub fn initialize_timezone_cache() {
        // TIMEZONE_CHOICE_DATAにアクセスして初期化を強制
        let _count = TIMEZONE_CHOICE_DATA.len();
        info!("タイムゾーンキャッシュを初期化しました（{}件）", _count);
    }
}
