use crate::facades::recruitment::button_handler;
use crate::types::{PoiseData, Result};
use poise::serenity_prelude::{ComponentInteraction, Context};
use tracing::{debug, error, info};

/// ComponentInteractionイベントハンドラ
///
/// ボタンクリックなどのコンポーネントインタラクションを処理
pub async fn on_component_interaction(
    ctx: &Context,
    interaction: &ComponentInteraction,
    data: &PoiseData,
) -> Result<()> {
    let custom_id = &interaction.data.custom_id;
    debug!(custom_id = %custom_id, "ComponentInteraction受信");

    // 募集ボタンのクリックを処理
    if custom_id.starts_with("recruit_") {
        // 即座にdeferして処理時間を確保（DB操作があるため）
        interaction.defer_ephemeral(&ctx.http).await.map_err(|e| {
            error!(error = %e, "defer_ephemeralに失敗しました");
            crate::types::AppError::Discord(e)
        })?;

        info!(custom_id = %custom_id, "募集ボタンのクリックを検出");

        match button_handler::handle_recruitment_button(ctx, interaction, &data.app_state).await {
            Ok(_) => {
                info!("募集ボタンの処理が正常に完了しました");
            }
            Err(e) => {
                error!(error = %e, "募集ボタンの処理中にエラーが発生しました");
                // エラーはFacade層で既にユーザーに通知済み
            }
        }
    } else {
        debug!(custom_id = %custom_id, "未対応のcomponent interaction");
    }

    Ok(())
}
