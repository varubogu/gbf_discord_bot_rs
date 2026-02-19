use std::sync::Arc;
use tracing::error;

use sea_orm::TransactionTrait;

use crate::infrastructure::database::session::set_current_guild_id;
use crate::services::channel::ChannelManagementService;
use crate::types::{AppError, Result, app_state::AppState};

/// ギルド管理ファサード
/// - 新規ギルド登録など、ギルド管理に関するユースケースを扱う
/// - トランザクション境界とRLS設定の一元管理を行う
pub struct GuildManagementFacade {
    app_state: Arc<AppState>,
}

impl GuildManagementFacade {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }

    /// 新規ギルドの登録（存在しなければ作成、既存なら何もしない）
    pub async fn register_new_guild(&self, guild_id: i64, guild_name: &str) -> Result<()> {
        let conn = self.app_state.guild_db();
        let txn = conn.begin().await?;

        // RLS適用
        set_current_guild_id(&txn, guild_id).await?;

        let result = async {
            let guild_repo = self.app_state.repositories.guild;
            let channel_type_repo = self.app_state.repositories.channel_type;
            let guild_channel_repo = self.app_state.repositories.guild_channel;
            let service =
                ChannelManagementService::new(guild_repo, channel_type_repo, guild_channel_repo);
            // 既存なければ作成、あれば更新
            service
                .register_guild(&txn, guild_id, guild_name.to_string())
                .await?;
            Ok::<_, AppError>(())
        }
        .await;

        match result {
            Ok(_) => {
                txn.commit().await?;
                Ok(())
            }
            Err(e) => {
                error!(error = %e, guild_id, "新規ギルド登録に失敗しました");
                txn.rollback().await?;
                Err(e)
            }
        }
    }
}
