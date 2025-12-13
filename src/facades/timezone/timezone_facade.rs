use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::repository::database::guild_timezone_repository::GuildTimezoneRepository;
use crate::services::timezone_service::TimezoneService;
use crate::types::app_state::AppState;
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
