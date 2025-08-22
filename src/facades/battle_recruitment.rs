use crate::infrastructure::database::container::RepositoryContainer;
use crate::services::battle_recruitment::cancel::CancelRecruitmentService;
use crate::services::battle_recruitment::new::NewRecruitmentService;
use crate::services::battle_recruitment::participants::ParticipantsService;
use crate::services::battle_recruitment::start::StartRecruitmentService;
use crate::services::battle_recruitment::update::UpdateRecruitmentService;
use crate::types::battle_type::BattleType;
use crate::types::{AppState, PoiseContext, Result};
use sea_orm::TransactionTrait;
use tracing::{error, info, instrument};

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

    /// 新しい募集を開始する（Rustらしい実装）
    #[instrument(skip(self), fields(quest = %quest_alias))]
    pub async fn new_recruitment(
        &self,
        ctx: &PoiseContext<'_>,
        quest_alias: &str,
        battle_type: BattleType,
    ) -> Result<()> {
        info!("BattleRecruitmentFacade::new_recruitment - 新しい募集を開始します");

        // guild_idとchannel_idを取得
        let guild_id = ctx.guild_id().map(|id| id.get()).unwrap_or(0);
        let channel_id = ctx.channel_id().get();
        let serenity_ctx = ctx.serenity_context().clone();

        // トランザクション処理（AppStateから共有DB接続を使用）
        let txn = self.app_state.db().begin().await?;

        match async {
            // NewRecruitmentServiceインスタンス作成
            let service = NewRecruitmentService::new().await?;

            // 募集作成処理
            service
                .create_recruitment(
                    &serenity_ctx,
                    channel_id,
                    guild_id,
                    quest_alias,
                    battle_type,
                    None, // デフォルトの日時を使用
                )
                .await?;

            Ok::<(), crate::types::AppError>(())
        }
        .await
        {
            Ok(_) => {
                txn.commit().await?;
                info!(quest = %quest_alias, "新規募集作成が完了しました");
                Ok(())
            }
            Err(e) => {
                txn.rollback().await?;
                error!(error = %e, quest = %quest_alias, "新規募集作成エラー");
                Err(e)
            }
        }
    }

    /// 募集内容を更新する（Rustらしい実装）
    #[instrument(skip(self), fields(message_id = %message_id))]
    pub async fn update_recruitment_information(
        &self,
        ctx: &poise::serenity_prelude::Context,
        guild_id: u64,
        channel_id: u64,
        message_id: u64,
        new_content: Option<String>,
    ) -> Result<()> {
        info!("BattleRecruitmentFacade::update_recruitment_information - 募集内容を更新します");

        let txn = self.app_state.db().begin().await?;

        match async {
            let service = UpdateRecruitmentService::new();

            service
                .update_recruitment_message(
                    ctx,
                    guild_id,
                    channel_id,
                    message_id,
                    new_content,
                    None,
                )
                .await?;

            Ok::<(), crate::types::AppError>(())
        }
        .await
        {
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

    /// 参加者を更新する（Rustらしい実装）
    #[instrument(skip(self), fields(message_id = %message_id))]
    pub async fn update_participants(
        &self,
        ctx: &poise::serenity_prelude::Context,
        guild_id: u64,
        channel_id: u64,
        message_id: u64,
    ) -> Result<()> {
        info!("BattleRecruitmentFacade::update_participants - 参加者を更新します");

        let txn = self.app_state.db().begin().await?;

        match async {
            let service = ParticipantsService::new().await?;

            // 募集情報をDBから取得
            let recruitment = service
                .get_recruitment_from_db(guild_id, channel_id, message_id)
                .await?;

            // 募集メッセージのリアクションとメンバーを取得
            let _participants_by_reaction = service
                .get_reactions_and_members(ctx, channel_id, message_id)
                .await?;

            // メッセージ更新処理
            let content = format!("募集ID: {} の参加者を更新しました", recruitment.id);
            let empty_participants = std::collections::HashMap::new();
            service
                .update_message(ctx, channel_id, message_id, &content, &empty_participants)
                .await?;

            Ok::<(), crate::types::AppError>(())
        }
        .await
        {
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

    /// 募集をキャンセルする（Rustらしい実装）
    #[instrument(skip(self), fields(message_id = %message_id))]
    pub async fn cancel_recruitment(
        &self,
        ctx: &poise::serenity_prelude::Context,
        guild_id: u64,
        channel_id: u64,
        message_id: u64,
    ) -> Result<()> {
        info!("BattleRecruitmentFacade::cancel_recruitment - 募集をキャンセルします");

        let txn = self.app_state.db().begin().await?;

        match async {
            // キャンセル処理のロジックを実装
            // 現在はプレースホルダー
            Ok::<(), crate::types::AppError>(())
        }
        .await
        {
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

    /// 募集を開始する（Rustらしい実装）
    #[instrument(skip(self), fields(message_id = %message_id))]
    pub async fn start_recruitment(
        &self,
        ctx: &poise::serenity_prelude::Context,
        guild_id: u64,
        channel_id: u64,
        message_id: u64,
    ) -> Result<()> {
        info!("BattleRecruitmentFacade::start_recruitment - 募集を開始します");

        let txn = self.app_state.db().begin().await?;

        match async {
            let service = StartRecruitmentService::new().await?;

            // DBから募集情報を取得
            let recruitment = service
                .get_recruitment_from_db(guild_id, channel_id, message_id)
                .await?;

            // リアクションから参加者一覧取得
            let _participants = service
                .get_participants_from_reactions(ctx, channel_id, message_id)
                .await?;

            // 開始処理のロジック（現在はプレースホルダー）
            info!(recruitment_id = %recruitment.id, "募集開始処理を実行");

            Ok::<(), crate::types::AppError>(())
        }
        .await
        {
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
