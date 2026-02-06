use crate::facades::recruitment::button_handler;
use crate::gateway::PoiseDiscordGateway;
use crate::types::discord::{DiscordChannelId, DiscordGuildId, DiscordMessageId};
use crate::types::{PoiseData, Result};
use poise::serenity_prelude::{ComponentInteraction, Context};
use std::sync::Arc;
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

    // 自動募集クエスト選択確認ボタンの処理
    if custom_id.starts_with("auto_quest_selection_check:") {
        use crate::events::interactions::components::auto_recruit_selection_check_handler;

        info!(custom_id = %custom_id, "自動募集クエスト選択確認ボタンを検出");

        match auto_recruit_selection_check_handler::handle_selection_check_button(
            ctx,
            interaction,
            data,
        )
        .await
        {
            Ok(_) => {
                info!("自動募集クエスト選択確認ボタンの処理が正常に完了しました");
            }
            Err(e) => {
                error!(error = %e, "自動募集クエスト選択確認ボタンの処理中にエラーが発生しました");
            }
        }
    }
    // 自動募集クエスト参加ボタンの処理（1クエスト1メッセージ形式）
    else if custom_id.starts_with("auto_quest_join:") {
        use crate::events::interactions::components::auto_recruit_quest_join_handler;

        info!(custom_id = %custom_id, "自動募集クエスト参加ボタンを検出");

        match auto_recruit_quest_join_handler::handle_quest_join_button(ctx, interaction, data)
            .await
        {
            Ok(_) => {
                info!("自動募集クエスト参加ボタンの処理が正常に完了しました");
            }
            Err(e) => {
                error!(error = %e, "自動募集クエスト参加ボタンの処理中にエラーが発生しました");
            }
        }
    }
    // 自動募集属性選択の処理（1クエスト1メッセージ形式）
    else if custom_id.starts_with("auto_quest_element:") {
        use crate::events::interactions::components::auto_recruit_element_handler;

        info!(custom_id = %custom_id, "自動募集属性選択を検出");

        match auto_recruit_element_handler::handle_element_selection(ctx, interaction, data).await {
            Ok(_) => {
                info!("自動募集属性選択の処理が正常に完了しました");
            }
            Err(e) => {
                error!(error = %e, "自動募集属性選択の処理中にエラーが発生しました");
            }
        }
    }
    // 自動募集クエスト選択の処理（レガシー：セレクトメニュー形式）
    else if custom_id.starts_with("auto_quest_select:") {
        use crate::events::interactions::components::auto_recruit_quest_handler;

        info!(custom_id = %custom_id, "自動募集クエスト選択を検出");

        match auto_recruit_quest_handler::handle_quest_selection_interaction(ctx, interaction, data)
            .await
        {
            Ok(_) => {
                info!("自動募集クエスト選択の処理が正常に完了しました");
            }
            Err(e) => {
                error!(error = %e, "自動募集クエスト選択の処理中にエラーが発生しました");
            }
        }
    }
    // 自動募集時間選択の処理
    else if custom_id.starts_with("auto_time_select:") {
        use crate::events::interactions::components::auto_recruit_time_handler;

        info!(custom_id = %custom_id, "自動募集時間選択を検出");

        match auto_recruit_time_handler::handle_time_selection_interaction(ctx, interaction, data)
            .await
        {
            Ok(_) => {
                info!("自動募集時間選択の処理が正常に完了しました");
            }
            Err(e) => {
                error!(error = %e, "自動募集時間選択の処理中にエラーが発生しました");
            }
        }
    }
    // 募集変更のセレクトメニューを処理
    else if custom_id.starts_with("recruit_change_") {
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

        // interactionからドメイン型を抽出
        let guild_id = match interaction.guild_id {
            Some(gid) => DiscordGuildId::new(gid.get()),
            None => {
                error!("ギルドIDが取得できませんでした");
                if let Err(e) = interaction
                    .edit_response(
                        &ctx.http,
                        poise::serenity_prelude::EditInteractionResponse::new()
                            .content("❌ エラー: サーバー内でのみ使用できます"),
                    )
                    .await
                {
                    error!(error = %e, "エラーメッセージの送信に失敗しました");
                }
                return Ok(());
            }
        };
        let channel_id = DiscordChannelId::new(interaction.channel_id.get());
        let message_id = DiscordMessageId::new(interaction.message.id.get());
        let user_id = interaction.user.id.get();

        // Gatewayを作成
        let gateway = PoiseDiscordGateway::new(Arc::clone(&ctx.http));

        match button_handler::handle_recruitment_select_menu(
            &gateway,
            &data.app_state,
            guild_id,
            channel_id,
            message_id,
            user_id,
            element_ids,
        )
        .await
        {
            Ok(result) => {
                info!("属性セレクトメニューの処理が正常に完了しました");
                // events層でedit_responseを呼び出す
                if let Err(e) = interaction
                    .edit_response(
                        &ctx.http,
                        poise::serenity_prelude::EditInteractionResponse::new()
                            .content(&result.message),
                    )
                    .await
                {
                    error!(error = %e, "応答メッセージの送信に失敗しました");
                }
            }
            Err(e) => {
                error!(error = %e, "属性セレクトメニューの処理中にエラーが発生しました");
                // エラーメッセージをedit_responseで送信
                if let Err(edit_err) = interaction
                    .edit_response(
                        &ctx.http,
                        poise::serenity_prelude::EditInteractionResponse::new()
                            .content(format!("❌ エラー: {}", e.user_message())),
                    )
                    .await
                {
                    error!(error = %edit_err, "エラーメッセージの送信に失敗しました");
                }
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

        // interactionからドメイン型を抽出
        let guild_id = match interaction.guild_id {
            Some(gid) => DiscordGuildId::new(gid.get()),
            None => {
                error!("ギルドIDが取得できませんでした");
                if let Err(e) = interaction
                    .edit_response(
                        &ctx.http,
                        poise::serenity_prelude::EditInteractionResponse::new()
                            .content("❌ エラー: サーバー内でのみ使用できます"),
                    )
                    .await
                {
                    error!(error = %e, "エラーメッセージの送信に失敗しました");
                }
                return Ok(());
            }
        };
        let channel_id = DiscordChannelId::new(interaction.channel_id.get());
        let message_id = DiscordMessageId::new(interaction.message.id.get());
        let user_id = interaction.user.id.get();

        // Gatewayを作成
        let gateway = PoiseDiscordGateway::new(Arc::clone(&ctx.http));

        match button_handler::handle_recruitment_button(
            &gateway,
            &data.app_state,
            guild_id,
            channel_id,
            message_id,
            user_id,
            custom_id,
        )
        .await
        {
            Ok(result) => {
                info!("募集ボタンの処理が正常に完了しました");
                // events層でedit_responseを呼び出す
                if let Err(e) = interaction
                    .edit_response(
                        &ctx.http,
                        poise::serenity_prelude::EditInteractionResponse::new()
                            .content(&result.message),
                    )
                    .await
                {
                    error!(error = %e, "応答メッセージの送信に失敗しました");
                }
            }
            Err(e) => {
                error!(error = %e, "募集ボタンの処理中にエラーが発生しました");
                // エラーメッセージをedit_responseで送信
                if let Err(edit_err) = interaction
                    .edit_response(
                        &ctx.http,
                        poise::serenity_prelude::EditInteractionResponse::new()
                            .content(format!("❌ エラー: {}", e.user_message())),
                    )
                    .await
                {
                    error!(error = %edit_err, "エラーメッセージの送信に失敗しました");
                }
            }
        }
    } else {
        debug!(custom_id = %custom_id, "未対応のcomponent interaction");
    }

    Ok(())
}
