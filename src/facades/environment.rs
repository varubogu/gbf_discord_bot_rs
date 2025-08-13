use crate::services::permission::has_bot_control_permission;
use crate::services::environment::service::load_environment_from_database;
use crate::repository::Database;
use crate::types::{PoiseContext, PoiseError};
use std::sync::Arc;

pub(crate) async fn load(ctx: &PoiseContext<'_>) -> Result<(), PoiseError>{

    // コマンド実行者の情報取得
    let member = ctx.author_member().await.unwrap();

    // 権限チェック
    let has_permission_result = has_bot_control_permission(ctx, &member).await;
    if let Err(permission_error) = has_permission_result {
        return Err(permission_error.into());
    }

    // データベース接続
    let db = match Database::new().await {
        Ok(database) => Arc::new(database),
        Err(e) => {
            let error_msg = format!("データベース接続エラー: {}", e);
            ctx.send(poise::CreateReply::default()
                .content(&error_msg)
                .ephemeral(true)
            ).await?;
            return Err(error_msg.into());
        }
    };

    // 環境変数読み込み処理（データベースから読み込み）
    match load_environment_from_database(db).await.map_err(|e| format!("環境変数読み込みエラー: {}", e)) {
        Ok(_) => {
            // 完了したことをメッセージで表示
            ctx.send(poise::CreateReply::default()
                .content("環境変数の読み込みが完了しました。")
                .ephemeral(true)
            ).await?;
            Ok(())
        },
        Err(error_msg) => {
            ctx.send(poise::CreateReply::default()
                .content(&error_msg)
                .ephemeral(true)
            ).await?;
            Err(error_msg.into())
        }
    }
}

