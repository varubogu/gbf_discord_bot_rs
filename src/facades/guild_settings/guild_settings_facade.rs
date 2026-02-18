use crate::infrastructure::database::session::set_current_guild_id;
use crate::repository::GuildSettingsRepository;
use crate::services::timezone_service::{self, TimezoneService};
use crate::types::app_state::AppState;
use crate::types::discord::AutocompleteOption;
use crate::types::{AppError, Result};
use chrono_tz::Tz;
use sea_orm::TransactionTrait;
use std::sync::Arc;
use tracing::{error, info};

/// タイムゾーン設定結果
#[derive(Debug, Clone)]
pub struct TimezoneSetResult {
    pub timezone: Tz,
}

/// ギルド設定取得結果
#[derive(Debug, Clone)]
pub struct GuildSettingsResult {
    pub timezone: String,
    pub locale: String,
}

/// ギルド設定ファサード
///
/// ギルド設定（タイムゾーン・ロケール）のユースケースを管理するファサード。
/// トランザクション境界の管理を担当。
pub struct GuildSettingsFacade {
    app_state: Arc<AppState>,
}

impl GuildSettingsFacade {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }

    /// タイムゾーンのオートコンプリート候補を取得（DB不要）
    ///
    /// - 文字列 `partial` にマッチする IANA タイムゾーンの候補を最大25件返します。
    /// - トランザクションは不要なため、Facade内でのDB操作は行いません。
    pub fn get_timezones_for_autocomplete(&self, partial: &str) -> Vec<AutocompleteOption> {
        timezone_service::get_timezones_for_autocomplete(partial)
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

        // トランザクション開始（Facade層の責務）
        let conn = self.app_state.guild_db();
        let txn = conn.begin().await?;

        // RLSポリシーのためにセッション変数を設定
        set_current_guild_id(&txn, guild_id).await?;

        let result = async {
            let timezone_repo = self.app_state.repositories.guild_settings;
            let timezone_service = TimezoneService::new(timezone_repo);
            let timezone = timezone_service
                .get_guild_timezone_with_txn(&txn, guild_id)
                .await?;

            info!(
                guild_id = guild_id,
                timezone = %timezone,
                "タイムゾーン取得に成功しました"
            );

            Ok::<_, AppError>(timezone)
        }
        .await;

        match result {
            Ok(timezone) => {
                txn.commit().await?;
                Ok(timezone)
            }
            Err(e) => {
                txn.rollback().await?;
                Err(e)
            }
        }
    }

    /// ギルド設定を取得
    ///
    /// # 引数
    /// - `guild_id`: ギルドID
    ///
    /// # 戻り値
    /// ギルド設定（未設定の場合はNone）
    pub async fn get_guild_settings(&self, guild_id: i64) -> Result<Option<GuildSettingsResult>> {
        info!(guild_id = guild_id, "ギルド設定取得を開始します");

        // トランザクション開始（Facade層の責務）
        let conn = self.app_state.guild_db();
        let txn = conn.begin().await?;

        // RLSポリシーのためにセッション変数を設定
        set_current_guild_id(&txn, guild_id).await?;

        let result = async {
            let settings_repo = self.app_state.repositories.guild_settings;
            let settings = settings_repo
                .find_by_guild_id_with_txn(&txn, guild_id)
                .await?;

            info!(
                guild_id = guild_id,
                has_settings = settings.is_some(),
                "ギルド設定取得に成功しました"
            );

            Ok::<_, AppError>(settings.map(|s| GuildSettingsResult {
                timezone: s.timezone,
                locale: s.locale,
            }))
        }
        .await;

        match result {
            Ok(settings) => {
                txn.commit().await?;
                Ok(settings)
            }
            Err(e) => {
                txn.rollback().await?;
                Err(e)
            }
        }
    }

    /// タイムゾーンとロケールを設定
    ///
    /// # 引数
    /// - `guild_id`: ギルドID
    /// - `timezone_str`: タイムゾーン文字列（IANA形式）
    /// - `locale`: ロケール（ja または en）
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
        locale: &str,
    ) -> Result<TimezoneSetResult> {
        info!(
            guild_id = guild_id,
            timezone = timezone_str,
            locale = locale,
            "タイムゾーンとロケール設定を開始します"
        );

        // タイムゾーンバリデーション
        let timezone = timezone_service::validate_timezone(timezone_str)?;

        // トランザクション開始（Facade層の責務）
        let conn = self.app_state.guild_db();
        let txn = conn.begin().await?;

        // RLSポリシーのためにセッション変数を設定
        set_current_guild_id(&txn, guild_id).await?;

        let result = async {
            let timezone_repo = self.app_state.repositories.guild_settings;
            let timezone_service = TimezoneService::new(timezone_repo);

            // タイムゾーンとロケールをupsert
            timezone_service
                .set_guild_timezone(&txn, guild_id, timezone.name(), locale)
                .await?;

            info!(
                guild_id = guild_id,
                timezone = %timezone,
                locale = locale,
                "タイムゾーンとロケール設定が完了しました"
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
                    locale = locale,
                    "タイムゾーンとロケール設定に成功しました"
                );
                Ok(set_result)
            }
            Err(e) => {
                txn.rollback().await?;
                error!(
                    error = %e,
                    guild_id = guild_id,
                    timezone = timezone_str,
                    locale = locale,
                    "タイムゾーンとロケール設定に失敗しました"
                );
                Err(e)
            }
        }
    }
}
