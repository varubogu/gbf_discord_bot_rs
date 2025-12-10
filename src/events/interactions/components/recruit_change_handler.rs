/// セレクトメニュー方式の募集変更ハンドラー
///
/// コンパイルエラー修正のため、簡素化した実装
use crate::types::{AppError, PoiseData, Result};
use poise::serenity_prelude::{ComponentInteraction, Context};
use tracing::{error, info};

/// 募集変更関連のコンポーネントインタラクションを処理
pub async fn handle_recruit_change_interaction(
    ctx: &Context,
    interaction: &ComponentInteraction,
    data: &PoiseData,
) -> Result<()> {
    let custom_id = &interaction.data.custom_id;

    info!(custom_id = %custom_id, "募集変更インタラクションを処理");

    // TODO: セレクトメニューの実装を完成させる
    // 現在は暫定実装のため、エラーメッセージのみ返す

    interaction
        .create_response(
            &ctx.http,
            poise::serenity_prelude::CreateInteractionResponse::Message(
                poise::serenity_prelude::CreateInteractionResponseMessage::new()
                    .content("セレクトメニュー方式の募集変更機能は現在実装中です。\nスラッシュコマンド `/マルチバトル募集内容変更` をご利用ください。")
                    .ephemeral(true),
            ),
        )
        .await?;

    Ok(())
}

/// 出発日時のみを更新（公開関数：モーダルハンドラーから呼び出し可能）
pub async fn update_recruitment_date(
    _ctx: &Context,
    _data: &PoiseData,
    _guild_id: u64,
    _message_id: u64,
    _event_date: chrono::DateTime<chrono::Utc>,
) -> Result<()> {
    // TODO: 実装を完成させる
    Err(AppError::Generic("日時変更機能は現在実装中です".to_string()))
}
