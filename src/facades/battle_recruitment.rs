use crate::infrastructure::database::container::RepositoryContainer;
use crate::services::battle_recruitment::cancel::CancelRecruitmentService;
use crate::services::battle_recruitment::get::GetRecruitmentService;
use crate::services::battle_recruitment::new::NewRecruitmentService;
use crate::services::battle_recruitment::participants::ParticipantsService;
use crate::services::battle_recruitment::start::StartRecruitmentService;
use crate::services::battle_recruitment::update::UpdateRecruitmentService;
use crate::repository::BattleRecruitmentRepository;
use crate::types::battle_type::BattleType;
use crate::types::{AppState, PoiseContext, Result, DiscordOperation, DiscordOperationResult};
use sea_orm::TransactionTrait;
use tracing::{error, info, instrument};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// BattleRecruitmentFacade - バトル募集に関するファサード（Rustらしいパターン）
///
/// AppStateパターンを使用して、共有状態を効率的に管理します。
/// 従来のDIコンテナではなく、Rustエコシステムに適した設計を採用。
#[derive(Debug)]
pub struct BattleRecruitmentFacade<'a> {
    app_state: &'a AppState,
}

impl<'a> BattleRecruitmentFacade<'a> {
    /// 新しいBattleRecruitmentFacadeを作成
    ///
    /// # 引数
    /// * `app_state` - アプリケーションの共有状態への参照
    ///
    /// # 戻り値
    /// 新しいBattleRecruitmentFacadeインスタンス
    pub fn new(app_state: &'a AppState) -> Self {
        Self { app_state }
    }

    /// 新しい募集を開始する（クロージャパターン）
    #[instrument(skip(self, discord_operation), fields(quest = %quest_alias))]
    pub async fn new_recruitment<F>(
        &self,
        quest_alias: &str,
        battle_type: BattleType,
        channel_id: u64,
        guild_id: u64,
        mut discord_operation: F,
    ) -> Result<u64>
    where
        F: FnMut(DiscordOperation) -> Pin<Box<dyn Future<Output=Result<DiscordOperationResult>> + Send>>,
    {
        info!("BattleRecruitmentFacade::new_recruitment - 新しい募集を開始します");

        // トランザクション処理（AppStateから共shared DB接続を使用）
        let txn = self.app_state.db().begin().await?;

        let result = async {
            // Repository作成（正しいコンストラクタを使用）
            let repos = RepositoryContainer::new(self.app_state.db());
            let battle_recruitment_repo = repos.battle_recruitment();
            let quest_repo = repos.quest();

            // NewRecruitmentServiceインスタンス作成（依存性注入）
            let service = NewRecruitmentService::new(battle_recruitment_repo, quest_repo);

            // 1. 募集データを作成（純粋なビジネスロジック）
            let recruitment_data = service
                .create_recruitment_data(quest_alias, battle_type, channel_id, guild_id, None)
                .await?;

            // 2. Discord操作を要求（副作用を外部に委譲）
            let discord_result = discord_operation(DiscordOperation::SendMessage {
                channel_id: recruitment_data.channel_id,
                content: recruitment_data.message_content.clone(),
                embed: Some(recruitment_data.embed.clone()),
            })
                .await?;

            // 3. リアクション追加要求（FnMutで複数回呼び出し可能）
            if let Some(message) = discord_result.message.clone() {
                for reaction in &recruitment_data.reactions {
                    discord_operation(DiscordOperation::AddReaction {
                        message: message.clone(),
                        emoji: reaction.clone(),
                    })
                        .await?;
                }
            }

            // 4. データベースに保存
            service
                .save_recruitment(&recruitment_data, discord_result.message_id)
                .await?;

            Ok::<u64, crate::types::AppError>(discord_result.message_id)
        }
            .await;

        match result {
            Ok(message_id) => {
                txn.commit().await?;
                info!(quest = %quest_alias, message_id = %message_id, "新規募集作成が完了しました");
                Ok(message_id)
            }
            Err(e) => {
                txn.rollback().await?;
                error!(error = %e, quest = %quest_alias, "新規募集作成エラー");
                Err(e)
            }
        }
    }

    /// 募集内容を更新する（クロージャパターン）
    #[instrument(skip(self, discord_operation), fields(message_id = %message_id))]
    pub async fn update_recruitment_information<F>(
        &self,
        guild_id: u64,
        channel_id: u64,
        message_id: u64,
        new_content: Option<String>,
        new_embed: Option<poise::serenity_prelude::CreateEmbed>,
        mut discord_operation: F,
    ) -> Result<()>
    where
        F: FnMut(DiscordOperation) -> Pin<Box<dyn Future<Output=Result<DiscordOperationResult>> + Send>>,
    {
        info!("BattleRecruitmentFacade::update_recruitment_information - 募集内容を更新します");

        let txn = self.app_state.db().begin().await?;

        let result = async {
            // Repository作成（service経由でアクセス）
            let repos = RepositoryContainer::new(self.app_state.db());
            let battle_recruitment_repo = Arc::new(repos.battle_recruitment().clone());

            // GetRecruitmentServiceを作成（依存性注入）
            let get_service = GetRecruitmentService::new(battle_recruitment_repo);

            // 募集情報の存在確認（service経由）
            let _recruitment = get_service
                .get_by_message(guild_id, channel_id, message_id)
                .await
                .map_err(|e| crate::types::AppError::Generic(e))?
                .ok_or_else(|| crate::types::AppError::NotFound("募集が見つかりませんでした".to_string()))?;

            // Discord操作を要求（メッセージ編集）
            discord_operation(DiscordOperation::EditMessage {
                channel_id,
                message_id,
                content: new_content,
                embed: new_embed,
            })
                .await?;

            Ok::<(), crate::types::AppError>(())
        }
            .await;

        match result {
            Ok(_) => {
                txn.commit().await?;
                info!(message_id = %message_id, "募集内容更新が完了しました");
                Ok(())
            }
            Err(e) => {
                txn.rollback().await?;
                error!(error = %e, message_id = %message_id, "募集内容更新エラー");
                Err(e)
            }
        }
    }

    /// 参加者を更新する（クロージャパターン）
    #[instrument(skip(self, discord_operation), fields(message_id = %message_id))]
    pub async fn update_participants<F>(
        &self,
        guild_id: u64,
        channel_id: u64,
        message_id: u64,
        mut discord_operation: F,
    ) -> Result<()>
    where
        F: FnMut(DiscordOperation) -> Pin<Box<dyn Future<Output=Result<DiscordOperationResult>> + Send>>,
    {
        info!("BattleRecruitmentFacade::update_participants - 参加者を更新します");

        let txn = self.app_state.db().begin().await?;

        let result = async {
            // Repository作成（Service層への依存性注入のため）
            let repos = RepositoryContainer::new(self.app_state.db());
            let battle_recruitment_repo = repos.battle_recruitment();

            // ParticipantsServiceを作成（依存性注入）
            let participants_service = ParticipantsService::new(Arc::new(battle_recruitment_repo.clone()));

            // Service層経由で募集情報を取得・更新
            let recruitment = participants_service
                .update_participants_by_message(guild_id, channel_id, message_id, &txn)
                .await?;

            // Discord操作を要求（メッセージ更新）
            let content = format!("募集ID: {} の参加者を更新しました", recruitment.id);
            discord_operation(DiscordOperation::EditMessage {
                channel_id,
                message_id,
                content: Some(content),
                embed: None,
            })
                .await?;

            Ok::<(), crate::types::AppError>(())
        }
            .await;

        match result {
            Ok(_) => {
                txn.commit().await?;
                info!(message_id = %message_id, "参加者更新が完了しました");
                Ok(())
            }
            Err(e) => {
                txn.rollback().await?;
                error!(error = %e, message_id = %message_id, "参加者更新エラー");
                Err(e)
            }
        }
    }

    /// 募集をキャンセルする（クロージャパターン）
    #[instrument(skip(self, discord_operation), fields(message_id = %message_id))]
    pub async fn cancel_recruitment<F>(
        &self,
        guild_id: u64,
        channel_id: u64,
        message_id: u64,
        mut discord_operation: F,
    ) -> Result<()>
    where
        F: FnMut(DiscordOperation) -> Pin<Box<dyn Future<Output=Result<DiscordOperationResult>> + Send>>,
    {
        info!("BattleRecruitmentFacade::cancel_recruitment - 募集をキャンセルします");

        let txn = self.app_state.db().begin().await?;

        let result = async {
            // Repository作成（Service層への依存性注入のため）
            let repos = RepositoryContainer::new(self.app_state.db());
            let battle_recruitment_repo = repos.battle_recruitment();

            // CancelRecruitmentServiceを作成（依存性注入）
            let cancel_service = CancelRecruitmentService::new(battle_recruitment_repo);

            // Service層経由でキャンセル処理
            let _recruitment = cancel_service
                .cancel_by_message(guild_id, channel_id, message_id, &txn)
                .await?;

            // Discord操作を要求（メッセージ削除）
            discord_operation(DiscordOperation::DeleteMessage {
                channel_id,
                message_id,
            })
                .await?;

            Ok::<(), crate::types::AppError>(())
        }
            .await;

        match result {
            Ok(_) => {
                txn.commit().await?;
                info!(message_id = %message_id, "募集キャンセルが完了しました");
                Ok(())
            }
            Err(e) => {
                txn.rollback().await?;
                error!(error = %e, message_id = %message_id, "募集キャンセルエラー");
                Err(e)
            }
        }
    }

    /// 募集を開始する（クロージャパターン）
    #[instrument(skip(self, discord_operation), fields(message_id = %message_id))]
    pub async fn start_recruitment<F>(
        &self,
        guild_id: u64,
        channel_id: u64,
        message_id: u64,
        mut discord_operation: F,
    ) -> Result<()>
    where
        F: FnMut(DiscordOperation) -> Pin<Box<dyn Future<Output=Result<DiscordOperationResult>> + Send>>,
    {
        info!("BattleRecruitmentFacade::start_recruitment - 募集を開始します");

        let txn = self.app_state.db().begin().await?;

        let result = async {
            // Repository作成（Service層への依存性注入のため）
            let repos = RepositoryContainer::new(self.app_state.db());
            let battle_recruitment_repo = repos.battle_recruitment();

            // StartRecruitmentServiceを作成（依存性注入）
            let start_service = StartRecruitmentService::new(Arc::new(battle_recruitment_repo.clone()));

            // Service層経由で開始処理
            let recruitment = start_service
                .start_by_message(guild_id, channel_id, message_id, &txn)
                .await?;

            // Discord操作を要求（メッセージ更新）
            let content = format!("募集ID: {} を開始しました", recruitment.id);
            discord_operation(DiscordOperation::EditMessage {
                channel_id,
                message_id,
                content: Some(content),
                embed: None,
            })
                .await?;

            Ok::<(), crate::types::AppError>(())
        }
            .await;

        match result {
            Ok(_) => {
                txn.commit().await?;
                info!(message_id = %message_id, "募集開始が完了しました");
                Ok(())
            }
            Err(e) => {
                txn.rollback().await?;
                error!(error = %e, message_id = %message_id, "募集開始エラー");
                Err(e)
            }
        }
    }
}

// 既存の関数型APIとの互換性を保つためのラッパー関数
// 段階的な移行のために一時的に保持

/// 新しい募集を開始する（互換性維持のための関数）
#[deprecated(note = "Use BattleRecruitmentFacade::new_recruitment instead")]
pub(crate) async fn new(
    ctx: &PoiseContext<'_>,
    quest_alias: &str,
    battle_type: BattleType,
) -> std::result::Result<(), String> {
    let app_state = &ctx.data().app_state;
    let facade = BattleRecruitmentFacade::new(app_state);

    facade
        .new_recruitment(ctx, quest_alias, battle_type)
        .await
        .map_err(|e| e.to_string())
}

/// 募集内容を更新する（互換性維持のための関数）
#[deprecated(note = "Use BattleRecruitmentFacade::update_recruitment_information instead")]
pub(crate) async fn information_update(
    ctx: &poise::serenity_prelude::Context,
    guild_id: u64,
    channel_id: u64,
    message_id: u64,
    new_content: Option<String>,
) -> std::result::Result<(), String> {
    // この関数は互換性のためのプレースホルダー
    // 実際のAppStateアクセスが必要
    Err("AppState access required".to_string())
}

/// 参加者を更新する（互換性維持のための関数）
#[deprecated(note = "Use BattleRecruitmentFacade::update_participants instead")]
pub(crate) async fn member_update(
    ctx: &poise::serenity_prelude::Context,
    guild_id: u64,
    channel_id: u64,
    message_id: u64,
) -> std::result::Result<(), String> {
    // この関数は互換性のためのプレースホルダー
    Err("AppState access required".to_string())
}

/// 募集をキャンセルする（互換性維持のための関数）
#[deprecated(note = "Use BattleRecruitmentFacade::cancel_recruitment instead")]
pub(crate) async fn cancel(
    ctx: &PoiseContext<'_>,
    guild_id: u64,
    channel_id: u64,
    message_id: u64,
) -> std::result::Result<(), String> {
    let app_state = &ctx.data().app_state;
    let facade = BattleRecruitmentFacade::new(app_state);

    facade
        .cancel_recruitment(ctx.serenity_context(), guild_id, channel_id, message_id)
        .await
        .map_err(|e| e.to_string())
}

/// 募集を開始する（互換性維持のための関数）
#[deprecated(note = "Use BattleRecruitmentFacade::start_recruitment instead")]
pub(crate) async fn start(
    ctx: &PoiseContext<'_>,
    guild_id: u64,
    channel_id: u64,
    message_id: u64,
) -> std::result::Result<(), String> {
    let app_state = &ctx.data().app_state;
    let facade = BattleRecruitmentFacade::new(app_state);

    facade
        .start_recruitment(ctx.serenity_context(), guild_id, channel_id, message_id)
        .await
        .map_err(|e| e.to_string())
}
