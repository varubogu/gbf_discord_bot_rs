use crate::infrastructure::database::db_helper::set_current_guild_id;
use crate::services::permission::has_bot_control_permission;
use crate::services::spreadsheet::guild_loader_service::{LoaderService, LoaderServiceImpl};
use crate::types::{PoiseContext, Result};
use sea_orm::TransactionTrait;
use tracing::{error, info, instrument};

/// サーバー固有スプレッドシートからデータを読み込み
///
/// # 引数
/// - `ctx`: PoiseContext
///
/// # 戻り値
/// - `Result<()>`: 処理結果
#[instrument(level = "debug", skip(ctx))]
pub async fn execute_load(ctx: &PoiseContext<'_>) -> Result<()> {
    info!("LoadFacade::execute_load - サーバー固有スプレッドシート読み込み処理を開始");

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

    let init_message = "サーバー固有スプレッドシートからデータ読み込み中...";
    ctx.say(init_message).await?;

    let guild_id = ctx.guild_id().map(|id| id.get()).unwrap_or(0);
    info!(
        "User {} started server-specific spreadsheet load in guild {}",
        ctx.author().id,
        guild_id
    );

    let app_state = &ctx.data().app_state;
    let txn = app_state.guild_db().begin().await?;

    // RLSポリシーのためにセッション変数を設定
    set_current_guild_id(&txn, guild_id as i64).await?;

    let result = async {
        // Service層のインスタンス化（静的ディスパッチ）
        let loader_service = LoaderServiceImpl::new();

        // スプレッドシート接続
        loader_service.open_spreadsheet(guild_id).await?;

        // サーバー固有データ読み込み
        let table_data = loader_service.load_table_data(guild_id).await?;

        // データ変換・検証
        let converted_data = loader_service.convert_data(table_data).await?;

        // データベース保存
        loader_service.save_data(converted_data, guild_id).await?;

        Ok(())
    }
    .await;

    match result {
        Ok(_) => {
            txn.commit().await?;
            ctx.say("サーバー固有スプレッドシートからデータ読み込み完了")
                .await?;
            info!(guild_id = %guild_id, "サーバー固有スプレッドシート読み込み処理完了");
            Ok(())
        }
        Err(e) => {
            txn.rollback().await?;
            error!(guild_id = %guild_id, error = %e, "サーバー固有スプレッドシート読み込み処理失敗");
            ctx.say("サーバー固有スプレッドシートからデータ読み込み失敗")
                .await?;
            Err(e)
        }
    }
}
