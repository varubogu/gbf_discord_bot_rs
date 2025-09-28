use crate::types::{PoiseContext, Result};
use crate::services::permission::is_bot_admin_server;
use crate::services::spreadsheet::global_loader_service::{GlobalLoaderService, GlobalLoaderServiceImpl};
use sea_orm::TransactionTrait;
use tracing::{error, info, instrument};

/// グローバルスプレッドシートからデータを読み込み
/// 
/// # 引数
/// - `ctx`: PoiseContext
/// 
/// # 戻り値
/// - `Result<()>`: 処理結果
#[instrument(level = "debug", skip(ctx))]
pub async fn execute_global_load(ctx: &PoiseContext<'_>) -> Result<()> {
    info!("GlobalLoadFacade::execute_global_load - グローバルスプレッドシート読み込み処理を開始");
    
    // 管理者専用サーバーかチェック
    let is_admin_server = is_bot_admin_server(ctx).await
        .map_err(|e| {
            error!("Admin server check failed: {}", e);
            e
        })?;
    
    if !is_admin_server {
        ctx.say("このコマンドは管理者専用サーバーでのみ実行可能です").await?;
        return Ok(());
    }
    
    let init_message = "グローバルスプレッドシートからデータ読み込み中...";
    ctx.say(init_message).await?;
    
    info!("User {} started global spreadsheet load in admin server", ctx.author().id);
    
    let app_state = &ctx.data().app_state;
    let txn = app_state.db().begin().await?;

    let result = async {
        // Service層のインスタンス化（静的ディスパッチ）
        let loader_service = GlobalLoaderServiceImpl::new();
        
        // スプレッドシート接続
        loader_service.open_spreadsheet().await?;
        
        // グローバルデータ読み込み
        let table_data = loader_service.load_global_table_data().await?;
        
        // データ変換・検証
        let converted_data = loader_service.convert_global_data(table_data).await?;
        
        // データベース保存
        loader_service.save_global_data(converted_data).await?;
        
        Ok(())
    }.await;

    match result {
        Ok(_) => {
            txn.commit().await?;
            ctx.say("グローバルスプレッドシートからデータ読み込み完了").await?;
            info!("グローバルスプレッドシート読み込み処理完了");
            Ok(())
        }
        Err(e) => {
            txn.rollback().await?;
            error!(error = %e, "グローバルスプレッドシート読み込み処理失敗");
            ctx.say("グローバルスプレッドシートからデータ読み込み失敗").await?;
            Err(e)
        }
    }
}
