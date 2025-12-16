use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::repository::database::guild_timezone_repository::GuildTimezoneRepository;
use crate::services::timezone_service::TimezoneService;
use crate::types::app_state::AppState;
use crate::types::{AppError, Result};
use chrono_tz::Tz;
use poise::serenity_prelude::AutocompleteChoice;
use sea_orm::TransactionTrait;
use std::sync::Arc;
use tracing::{error, info};

/// タイムゾーン設定結果
#[derive(Debug, Clone)]
pub struct TimezoneSetResult {
    pub timezone: Tz,
}

/// タイムゾーンファサード
///
/// タイムゾーン設定のユースケースを管理するファサード。
/// トランザクション境界の管理を担当。
pub struct TimezoneFacade {
    app_state: Arc<AppState>,
}

impl TimezoneFacade {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }

    /// タイムゾーンのオートコンプリート候補を取得（DB不要）
    ///
    /// - 文字列 `partial` にマッチする IANA タイムゾーンの候補を最大25件返します。
    /// - トランザクションは不要なため、Facade内でのDB操作は行いません。
    pub fn get_timezones_for_autocomplete(&self, partial: &str) -> Vec<AutocompleteChoice> {
        TimezoneService::get_timezones_for_autocomplete(partial)
    }

    /// タイムゾーンを取得
    ///
    /// # 引数
    /// - `guild_id`: ギルドID
    ///
    /// # 戻り値
    /// タイムゾーン（未設定の場合はAsia/Tokyo）
    pub async fn get_timezone(&self, guild_id: i64) -> Result<Tz> {
        info!(guild_id = guild_id, "タイムゾーン取得を開始します");

        let conn = self.app_state.guild_db();
        let timezone_repo = Arc::new(GuildTimezoneRepository::new());
        let timezone_service = TimezoneService::new(timezone_repo);

        let timezone = timezone_service.get_guild_timezone(conn, guild_id).await?;

        info!(
            guild_id = guild_id,
            timezone = %timezone,
            "タイムゾーン取得に成功しました"
        );

        Ok(timezone)
    }

    /// タイムゾーンを設定
    ///
    /// # 引数
    /// - `guild_id`: ギルドID
    /// - `timezone_str`: タイムゾーン文字列（IANA形式）
    ///
    /// # 戻り値
    /// タイムゾーン設定結果
    ///
    /// # トランザクション管理
    /// このメソッドはトランザクションを開始・コミット・ロールバックを管理します。
    pub async fn set_timezone(
        &self,
        guild_id: i64,
        timezone_str: &str,
    ) -> Result<TimezoneSetResult> {
        info!(
            guild_id = guild_id,
            timezone = timezone_str,
            "タイムゾーン設定を開始します"
        );

        // タイムゾーンバリデーション
        let timezone = TimezoneService::validate_timezone(timezone_str)?;

        // トランザクション開始（Facade層の責務）
        let conn = self.app_state.guild_db();
        let txn = conn.begin().await?;

        // RLSポリシーのためにセッション変数を設定
        set_current_guild_id(&txn, guild_id).await?;

        let result = async {
            let timezone_repo = Arc::new(GuildTimezoneRepository::new());

            // タイムゾーンをupsert
            timezone_repo
                .upsert_with_txn(&txn, guild_id, timezone.name())
                .await?;

            info!(
                guild_id = guild_id,
                timezone = %timezone,
                "タイムゾーン設定が完了しました"
            );

            Ok::<_, AppError>(TimezoneSetResult { timezone })
        }
        .await;

        // 結果に応じてcommit/rollback（Facade層の責務）
        match result {
            Ok(set_result) => {
                txn.commit().await?;
                info!(
                    guild_id = guild_id,
                    timezone = %set_result.timezone,
                    "タイムゾーン設定に成功しました"
                );
                Ok(set_result)
            }
            Err(e) => {
                txn.rollback().await?;
                error!(
                    error = %e,
                    guild_id = guild_id,
                    timezone = timezone_str,
                    "タイムゾーン設定に失敗しました"
                );
                Err(e)
            }
        }
    }
}
