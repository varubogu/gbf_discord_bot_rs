use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::repository::database::channel_type_repository::ChannelTypeRepository;
use crate::repository::database::guild_channel_repository::GuildChannelRepository;
use crate::repository::database::guild_repository::GuildRepository;
use crate::services::channel::channel_display_service::{
    ChannelDisplayService, ChannelSettingsDisplay,
};
use crate::services::channel::channel_type_query_service::ChannelTypeQueryService;
use crate::types::app_state::AppState;
use crate::types::{AppError, Result};
use poise::serenity_prelude::AutocompleteChoice;
use sea_orm::TransactionTrait;
use std::sync::Arc;
use tracing::{error, info};

/// チャンネル登録結果
#[derive(Debug, Clone)]
pub struct ChannelRegistrationResult {
    pub channel_type_name: String,
    pub channel_id: i64,
    pub settings_display: ChannelSettingsDisplay,
}

/// チャンネル登録解除結果
#[derive(Debug, Clone)]
pub struct ChannelUnregistrationResult {
    pub channel_type_name: String,
    pub old_channel_id: i64,
    pub settings_display: ChannelSettingsDisplay,
}

/// チャンネル管理ファサード
///
/// チャンネル登録・削除・表示などのユースケースを管理するファサード。
/// トランザクション境界の管理と複数サービスの協調を担当。
pub struct ChannelManagementFacade {
    app_state: Arc<AppState>,
}

impl ChannelManagementFacade {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }

    /// チャンネル種別のオートコンプリート候補を取得
    ///
    /// - DBの参照のみでトランザクションは不要。
    /// - Facade 層から Service を経由して Repository へアクセス。
    pub async fn get_channel_types_for_autocomplete(&self) -> Result<Vec<AutocompleteChoice>> {
        let conn = self.app_state.guild_db();
        let service = ChannelTypeQueryService::new();
        service.get_channel_types_for_autocomplete(conn).await
    }

    /// チャンネルを登録
    ///
    /// # 引数
    /// - `guild_id`: ギルドID
    /// - `guild_name`: ギルド名
    /// - `channel_type_id`: チャンネル種別ID
    /// - `channel_id`: チャンネルID
    ///
    /// # 戻り値
    /// チャンネル登録結果
    ///
    /// # トランザクション管理
    /// このメソッドはトランザクションを開始・コミット・ロールバックを管理します。
    pub async fn register_channel(
        &self,
        guild_id: i64,
        guild_name: String,
        channel_type_id: i32,
        channel_id: i64,
    ) -> Result<ChannelRegistrationResult> {
        info!(
            guild_id = guild_id,
            channel_type_id = channel_type_id,
            channel_id = channel_id,
            "チャンネル登録を開始します"
        );

        // トランザクション開始（Facade層の責務）
        let conn = self.app_state.guild_db();
        let txn = conn.begin().await?;

        // RLSポリシーのためにセッション変数を設定
        set_current_guild_id(&txn, guild_id).await?;

        let result = async {
            let guild_repo = GuildRepository::new();
            let channel_type_repo = ChannelTypeRepository::new();
            let guild_channel_repo = GuildChannelRepository::new();
            let display_service = ChannelDisplayService::new();

            // 1. ギルドが存在しない場合は自動登録
            guild_repo
                .upsert_with_txn(&txn, guild_id, guild_name)
                .await?;

            // 2. チャンネル種別が存在するか確認
            let channel_type_model = channel_type_repo
                .get_by_id(&txn, channel_type_id)
                .await?
                .ok_or_else(|| {
                    AppError::NotFound(format!(
                        "チャンネル種別ID {} が見つかりませんでした",
                        channel_type_id
                    ))
                })?;

            // 3. ギルドチャンネルを登録または更新
            guild_channel_repo
                .upsert_with_txn(&txn, guild_id, channel_type_id, channel_id)
                .await?;

            info!(
                guild_id = guild_id,
                channel_type_id = channel_type_id,
                channel_id = channel_id,
                "チャンネル登録が完了しました"
            );

            // 4. 設定状況を取得
            let settings_display = display_service.get_channel_settings(&txn, guild_id).await?;

            Ok::<_, AppError>(ChannelRegistrationResult {
                channel_type_name: channel_type_model.name.clone(),
                channel_id,
                settings_display,
            })
        }
        .await;

        // 結果に応じてcommit/rollback（Facade層の責務）
        match result {
            Ok(registration_result) => {
                txn.commit().await?;
                info!(
                    guild_id = guild_id,
                    channel_type_id = channel_type_id,
                    "チャンネル登録に成功しました"
                );
                Ok(registration_result)
            }
            Err(e) => {
                txn.rollback().await?;
                error!(
                    error = %e,
                    guild_id = guild_id,
                    channel_type_id = channel_type_id,
                    "チャンネル登録に失敗しました"
                );
                Err(e)
            }
        }
    }

    /// チャンネル登録を解除
    ///
    /// # 引数
    /// - `guild_id`: ギルドID
    /// - `channel_type_id`: チャンネル種別ID
    ///
    /// # 戻り値
    /// チャンネル登録解除結果
    ///
    /// # トランザクション管理
    /// このメソッドはトランザクションを開始・コミット・ロールバックを管理します。
    pub async fn unregister_channel(
        &self,
        guild_id: i64,
        channel_type_id: i32,
    ) -> Result<ChannelUnregistrationResult> {
        info!(
            guild_id = guild_id,
            channel_type_id = channel_type_id,
            "チャンネル登録解除を開始します"
        );

        // トランザクション開始（Facade層の責務）
        let conn = self.app_state.guild_db();
        let txn = conn.begin().await?;

        // RLSポリシーのためにセッション変数を設定
        set_current_guild_id(&txn, guild_id).await?;

        let result = async {
            let channel_type_repo = ChannelTypeRepository::new();
            let guild_channel_repo = GuildChannelRepository::new();
            let display_service = ChannelDisplayService::new();

            // 1. チャンネル種別が存在するか確認
            let channel_type_model = channel_type_repo
                .get_by_id(&txn, channel_type_id)
                .await?
                .ok_or_else(|| {
                    AppError::NotFound(format!(
                        "チャンネル種別ID {} が見つかりませんでした",
                        channel_type_id
                    ))
                })?;

            // 2. 削除前に現在の設定を取得
            let existing_channel = guild_channel_repo
                .get_by_guild_and_type_with_txn(&txn, guild_id, channel_type_id)
                .await?;

            let old_channel_id =
                existing_channel
                    .as_ref()
                    .map(|c| c.channel_id)
                    .ok_or_else(|| {
                        AppError::NotFound(format!(
                            "チャンネル種別「{}」の設定が見つかりませんでした",
                            channel_type_model.name
                        ))
                    })?;

            // 3. ギルドチャンネルを削除
            guild_channel_repo
                .delete_with_txn(&txn, guild_id, channel_type_id)
                .await?;

            info!(
                guild_id = guild_id,
                channel_type_id = channel_type_id,
                "チャンネル登録解除が完了しました"
            );

            // 4. 設定状況を取得
            let settings_display = display_service.get_channel_settings(&txn, guild_id).await?;

            Ok::<_, AppError>(ChannelUnregistrationResult {
                channel_type_name: channel_type_model.name.clone(),
                old_channel_id,
                settings_display,
            })
        }
        .await;

        // 結果に応じてcommit/rollback（Facade層の責務）
        match result {
            Ok(unregistration_result) => {
                txn.commit().await?;
                info!(
                    guild_id = guild_id,
                    channel_type_id = channel_type_id,
                    "チャンネル登録解除に成功しました"
                );
                Ok(unregistration_result)
            }
            Err(e) => {
                txn.rollback().await?;
                error!(
                    error = %e,
                    guild_id = guild_id,
                    channel_type_id = channel_type_id,
                    "チャンネル登録解除に失敗しました"
                );
                Err(e)
            }
        }
    }

    /// チャンネル設定を表示
    ///
    /// # 引数
    /// - `guild_id`: ギルドID
    ///
    /// # 戻り値
    /// チャンネル設定表示データ
    ///
    /// # トランザクション管理
    /// このメソッドはトランザクションを開始・コミット・ロールバックを管理します。
    pub async fn show_channel_settings(&self, guild_id: i64) -> Result<ChannelSettingsDisplay> {
        info!(guild_id = guild_id, "チャンネル設定表示を開始します");

        // トランザクション開始（Facade層の責務）
        let conn = self.app_state.guild_db();
        let txn = conn.begin().await?;

        // RLSポリシーのためにセッション変数を設定
        set_current_guild_id(&txn, guild_id).await?;

        let result = async {
            let display_service = ChannelDisplayService::new();

            // 設定状況を取得
            let settings_display = display_service.get_channel_settings(&txn, guild_id).await?;

            Ok::<_, AppError>(settings_display)
        }
        .await;

        // 結果に応じてcommit/rollback（Facade層の責務）
        match result {
            Ok(settings_display) => {
                txn.commit().await?;
                info!(guild_id = guild_id, "チャンネル設定表示に成功しました");
                Ok(settings_display)
            }
            Err(e) => {
                txn.rollback().await?;
                error!(
                    error = %e,
                    guild_id = guild_id,
                    "チャンネル設定表示に失敗しました"
                );
                Err(e)
            }
        }
    }
}
