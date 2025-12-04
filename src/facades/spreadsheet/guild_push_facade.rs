use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::services::permission::has_bot_control_permission;
use crate::services::spreadsheet::guild_push_service::{PushService, PushServiceImpl};
use crate::types::{PoiseContext, Result};
use sea_orm::TransactionTrait;
use tracing::{error, info, instrument};

/// サーバー固有スプレッドシートにデータを書き込み
///
/// # 引数
/// - `ctx`: PoiseContext
///
/// # 戻り値
/// - `Result<()>`: 処理結果
#[instrument(level = "debug", skip(ctx))]
pub async fn execute_push(ctx: &PoiseContext<'_>) -> Result<()> {
    info!("PushFacade::execute_push - サーバー固有スプレッドシート書き込み処理を開始");

    // コマンド実行者の情報取得
    let member = ctx
        .author_member()
        .await
        .ok_or("メンバー情報を取得できません".to_string())?;

    // gbf_bot_controlロール権限チェック
    let has_permission_result = has_bot_control_permission(ctx, &member).await;
    if let Err(permission_error) = has_permission_result {
        ctx.say(&format!("権限エラー: {}", permission_error))
            .await?;
        return Ok(());
    }

    let init_message = "データベースからサーバー固有スプレッドシートへ書き込み中...";
    ctx.say(init_message).await?;

    let guild_id = ctx.guild_id().map(|id| id.get()).unwrap_or(0);
    info!(
        "User {} started server-specific spreadsheet push in guild {}",
        ctx.author().id,
        guild_id
    );

    let app_state = &ctx.data().app_state;
    let txn = app_state.guild_db().begin().await?;

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id as i64).await?;

    let result = async {
        // Service層のインスタンス化（静的ディスパッチ）
        let push_service = PushServiceImpl::new();

        // スプレッドシート接続
        PushService::open_spreadsheet(&push_service, guild_id).await?;

        // サーバー固有データ取得
        let data = PushService::load_data(&push_service, guild_id).await?;

        // データ変換・検証
        let converted_data = PushService::convert_data(&push_service, data).await?;

        // スプレッドシート書き込み
        PushService::push_data(&push_service, converted_data, guild_id).await?;

        Ok(())
    }
    .await;

    match result {
        Ok(_) => {
            txn.commit().await?;
            ctx.say("データベースからサーバー固有スプレッドシートへ書き込み完了")
                .await?;
            info!(guild_id = %guild_id, "サーバー固有スプレッドシート書き込み処理完了");
            Ok(())
        }
        Err(e) => {
            txn.rollback().await?;
            error!(guild_id = %guild_id, error = %e, "サーバー固有スプレッドシート書き込み処理失敗");
            ctx.say("データベースからサーバー固有スプレッドシートへ書き込み失敗")
                .await?;
            Err(e)
        }
    }
}
