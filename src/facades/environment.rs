use crate::events::permission::has_bot_control_permission;
// use crate::services::environment::service::load_environment_from_database;
use crate::types::{PoiseContext, Result};

pub(crate) async fn load(ctx: &PoiseContext<'_>) -> Result<()> {
    // コマンド実行者の情報取得
    let member = ctx
        .author_member()
        .await
        .ok_or_else(|| crate::types::AppError::Config {
            message:
                "メンバー情報を取得できませんでした。このコマンドはサーバー内でのみ実行可能です"
                    .to_string(),
        })?;

    // 権限チェック
    let has_permission_result = has_bot_control_permission(ctx, &member).await;
    if let Err(permission_error) = has_permission_result {
        return Err(permission_error.into());
    }

    panic!();
    // // 環境変数読み込み処理（データベースから読み込み）
    // match load_environment_from_database(db).await.map_err(|e| format!("環境変数読み込みエラー: {}", e)) {
    //     Ok(_) => {
    //         // 完了したことをメッセージで表示
    //         ctx.send(poise::CreateReply::default()
    //             .content("環境変数の読み込みが完了しました。")
    //             .ephemeral(true)
    //         ).await?;
    //         Ok(())
    //     },
    //     Err(error_msg) => {
    //         ctx.send(poise::CreateReply::default()
    //             .content(&error_msg)
    //             .ephemeral(true)
    //         ).await?;
    //         Err(error_msg.into())
    //     }
    // }
}
