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

    // 募集変更のセレクトメニューを処理
    if custom_id.starts_with("recruit_change_") {
        use crate::events::interactions::components::recruit_change_handler;

        info!(custom_id = %custom_id, "募集変更インタラクションを検出");

        match recruit_change_handler::handle_recruit_change_interaction(ctx, interaction, data)
            .await
        {
            Ok(_) => {
                info!("募集変更の処理が正常に完了しました");
            }
            Err(e) => {
                error!(error = %e, "募集変更の処理中にエラーが発生しました");
                // エラーメッセージを表示
                let _ = interaction
                    .create_response(
                        &ctx.http,
                        poise::serenity_prelude::CreateInteractionResponse::Message(
                            poise::serenity_prelude::CreateInteractionResponseMessage::new()
                                .content(format!("エラー: {}", e.user_message()))
                                .ephemeral(true),
                        ),
                    )
                    .await;
            }
        }
    }
    // 属性セレクトメニューの処理（選択した属性で即座に参加処理）
    else if custom_id == "recruit_select_elements" {
        // セレクトメニューはボタンと同様にDB操作があるため、即座にdeferする
        interaction.defer_ephemeral(&ctx.http).await.map_err(|e| {
            error!(error = %e, "defer_ephemeralに失敗しました");
            crate::types::AppError::Discord(Box::new(e))
        })?;

        info!(custom_id = %custom_id, "属性セレクトメニューの選択を検出");

        // 選択された値を取得
        use poise::serenity_prelude::ComponentInteractionDataKind;
        let selected_values = match &interaction.data.kind {
            ComponentInteractionDataKind::StringSelect { values } => values.clone(),
            _ => {
                return Err(crate::types::AppError::Generic(
                    "予期しないコンポーネントタイプです".to_string(),
                ));
            }
        };

        debug!(selected_values = ?selected_values, "セレクトメニューで選択された値");

        // 選択値を属性IDリストに変換
        let element_ids: Vec<i32> = selected_values
            .iter()
            .filter_map(|s| s.parse().ok())
            .collect();

        if element_ids.is_empty() {
            // エラーメッセージを返す
            if let Err(e) = interaction
                .edit_response(
                    &ctx.http,
                    poise::serenity_prelude::EditInteractionResponse::new()
                        .content("❌ エラー: 属性を選択してください"),
                )
                .await
            {
                error!(error = %e, "エラーメッセージの送信に失敗しました");
            }
            return Ok(());
        }

        // button_handlerを使って参加処理を実行
        // ただし、セレクトメニューは複数の属性を一度に登録するため、
        // カスタムIDを動的に生成してhandle_recruitment_buttonを呼び出すのではなく、
        // 直接ロジックを実装する
        match button_handler::handle_recruitment_select_menu(
            ctx,
            interaction,
            &data.app_state,
            element_ids,
        )
        .await
        {
            Ok(_) => {
                info!("属性セレクトメニューの処理が正常に完了しました");
            }
            Err(e) => {
                error!(error = %e, "属性セレクトメニューの処理中にエラーが発生しました");
                // エラーはFacade層で既にユーザーに通知済み
            }
        }
    }
    // 募集ボタンのクリックを処理
    else if custom_id.starts_with("recruit_") {
        // 即座にdeferして処理時間を確保（DB操作があるため）
        interaction.defer_ephemeral(&ctx.http).await.map_err(|e| {
            error!(error = %e, "defer_ephemeralに失敗しました");
            crate::types::AppError::Discord(Box::new(e))
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
