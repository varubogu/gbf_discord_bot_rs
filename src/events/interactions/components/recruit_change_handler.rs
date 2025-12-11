use crate::repository::database::battle_style_repository::{
    BattleStyleRepository, SeaOrmBattleStyleRepository,
};
use crate::repository::database::quest_repository::SeaOrmQuestRepository;
use crate::repository::quests_repository::QuestRepository;
use crate::types::{AppError, PoiseData, Result};
use poise::serenity_prelude::{
    ComponentInteraction, ComponentInteractionDataKind, Context, CreateActionRow,
    CreateInputText, CreateInteractionResponse, CreateInteractionResponseMessage, CreateModal,
    CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption, InputTextStyle,
};
use tracing::{error, info};

/// 募集変更関連のコンポーネントインタラクションを処理
pub async fn handle_recruit_change_interaction(
    ctx: &Context,
    interaction: &ComponentInteraction,
    data: &PoiseData,
) -> Result<()> {
    let custom_id = &interaction.data.custom_id;

    if custom_id.starts_with("recruit_change_select_field:") {
        // 変更項目選択の処理
        handle_field_selection(ctx, interaction, data).await
    } else if custom_id.starts_with("recruit_change_quest:") {
        // クエスト選択の処理
        handle_quest_selection(ctx, interaction, data).await
    } else if custom_id.starts_with("recruit_change_style:") {
        // 攻略方法選択の処理
        handle_battle_style_selection(ctx, interaction, data).await
    } else {
        Ok(())
    }
}

/// 変更する項目の選択を処理
async fn handle_field_selection(
    ctx: &Context,
    interaction: &ComponentInteraction,
    data: &PoiseData,
) -> Result<()> {
    // メッセージIDを抽出
    let message_id = extract_message_id(&interaction.data.custom_id)?;

    // 選択された値を取得
    let selected_fields = match &interaction.data.kind {
        ComponentInteractionDataKind::StringSelect { values } => values.clone(),
        _ => {
            return Err(AppError::Generic(
                "予期しないコンポーネントタイプです".to_string(),
            ))
        }
    };

    info!(
        message_id = %message_id,
        fields = ?selected_fields,
        "変更項目が選択されました"
    );

    // 選択された項目に基づいて次のステップを決定
    if selected_fields.contains(&"quest".to_string()) {
        // クエスト選択メニューを表示
        show_quest_selection_menu(ctx, interaction, data, message_id).await
    } else if selected_fields.contains(&"battle_style".to_string()) {
        // 攻略方法選択メニューを表示
        show_battle_style_selection_menu(ctx, interaction, data, message_id).await
    } else if selected_fields.contains(&"date".to_string()) {
        // 日時入力モーダルを表示
        show_date_input_modal(ctx, interaction, message_id).await
    } else {
        Err(AppError::Generic("選択された項目がありません".to_string()))
    }
}

/// クエスト選択メニューを表示
async fn show_quest_selection_menu(
    ctx: &Context,
    interaction: &ComponentInteraction,
    data: &PoiseData,
    message_id: u64,
) -> Result<()> {
    // クエストリストを取得
    let quest_repo = SeaOrmQuestRepository::new();
    let quests = quest_repo.get_all(data.app_state.guild_db()).await?;

    // セレクトメニューのオプションを作成（最大25個まで）
    let options: Vec<CreateSelectMenuOption> = quests
        .iter()
        .take(25)
        .map(|quest| CreateSelectMenuOption::new(&quest.name, quest.id.to_string()))
        .collect();

    let custom_id = format!("recruit_change_quest:{}", message_id);
    let select_menu = CreateSelectMenu::new(custom_id, CreateSelectMenuKind::String { options })
        .placeholder("変更するクエストを選択してください");

    let components = vec![CreateActionRow::SelectMenu(select_menu)];

    interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new()
                    .content("変更するクエストを選択してください")
                    .components(components),
            ),
        )
        .await?;

    Ok(())
}

/// 攻略方法選択メニューを表示
async fn show_battle_style_selection_menu(
    ctx: &Context,
    interaction: &ComponentInteraction,
    data: &PoiseData,
    message_id: u64,
) -> Result<()> {
    // 攻略方法リストを取得
    let battle_style_repo = SeaOrmBattleStyleRepository::new();
    let battle_styles = battle_style_repo.get_all(data.app_state.guild_db()).await?;

    // セレクトメニューのオプションを作成
    let options: Vec<CreateSelectMenuOption> = battle_styles
        .iter()
        .map(|style| CreateSelectMenuOption::new(&style.display_name, style.id.to_string()))
        .collect();

    let custom_id = format!("recruit_change_style:{}", message_id);
    let select_menu = CreateSelectMenu::new(custom_id, CreateSelectMenuKind::String { options })
        .placeholder("攻略方法を選択してください");

    let components = vec![CreateActionRow::SelectMenu(select_menu)];

    interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new()
                    .content("攻略方法を選択してください")
                    .components(components),
            ),
        )
        .await?;

    Ok(())
}

/// 日時入力モーダルを表示
async fn show_date_input_modal(
    ctx: &Context,
    interaction: &ComponentInteraction,
    message_id: u64,
) -> Result<()> {
    let custom_id = format!("recruit_change_date_modal:{}", message_id);

    let modal = CreateModal::new(custom_id, "出発日時変更").components(vec![
        CreateActionRow::InputText(
            CreateInputText::new(InputTextStyle::Short, "出発日時", "event_date")
                .placeholder("例: 12/25 22:30")
                .required(true),
        ),
    ]);

    interaction
        .create_response(&ctx.http, CreateInteractionResponse::Modal(modal))
        .await?;

    Ok(())
}

/// クエスト選択を処理
async fn handle_quest_selection(
    ctx: &Context,
    interaction: &ComponentInteraction,
    data: &PoiseData,
) -> Result<()> {
    let message_id = extract_message_id(&interaction.data.custom_id)?;

    // 選択された値を取得
    let quest_name = match &interaction.data.kind {
        ComponentInteractionDataKind::StringSelect { values } => values
            .first()
            .ok_or_else(|| AppError::Generic("クエストが選択されていません".to_string()))?,
        _ => {
            return Err(AppError::Generic(
                "予期しないコンポーネントタイプです".to_string(),
            ))
        }
    };

    // quest_nameは実はquest_idの文字列なので、IDからクエスト名を取得
    let quest_id: i32 = quest_name
        .parse()
        .map_err(|_| AppError::Generic("クエストIDの解析に失敗しました".to_string()))?;

    let quest_repo = SeaOrmQuestRepository::new();
    let quest = quest_repo
        .get_by_target_id(data.app_state.guild_db(), quest_id)
        .await?
        .ok_or_else(|| AppError::Generic("クエストが見つかりません".to_string()))?;

    info!(
        message_id = %message_id,
        quest_name = %quest.name,
        "クエストが選択されました"
    );

    // Deferして処理時間を確保
    interaction.defer(&ctx.http).await?;

    // 対象メッセージを取得
    let channel_id = interaction.channel_id;
    let guild_id = interaction
        .guild_id
        .ok_or_else(|| AppError::Generic("ギルドIDが取得できません".to_string()))?
        .get();

    let target_message = channel_id
        .message(&ctx.http, message_id)
        .await
        .map_err(|e| {
            error!(error = %e, "メッセージの取得に失敗しました");
            AppError::Generic("対象のメッセージが見つかりませんでした".to_string())
        })?;

    // リファクタリングされたfacadeを呼び出す
    let result = crate::facades::recruitment::change::change_recruitment_information_internal(
        &data.app_state,
        &ctx.http,
        guild_id,
        &target_message,
        Some(&quest.name),
        None,
        None,
    )
    .await;

    match result {
        Ok(_) => {
            interaction
                .edit_response(
                    &ctx.http,
                    poise::serenity_prelude::EditInteractionResponse::new()
                        .content(format!("クエストを「{}」に変更しました。", quest.name))
                        .components(vec![]),
                )
                .await?;
        }
        Err(e) => {
            error!(error = %e, "クエスト変更に失敗しました");
            interaction
                .edit_response(
                    &ctx.http,
                    poise::serenity_prelude::EditInteractionResponse::new()
                        .content(format!("エラー: {}", e.user_message()))
                        .components(vec![]),
                )
                .await?;
        }
    }

    Ok(())
}

/// 攻略方法選択を処理
async fn handle_battle_style_selection(
    ctx: &Context,
    interaction: &ComponentInteraction,
    data: &PoiseData,
) -> Result<()> {
    let message_id = extract_message_id(&interaction.data.custom_id)?;

    // 選択された値を取得
    let battle_style_id_str = match &interaction.data.kind {
        ComponentInteractionDataKind::StringSelect { values } => values
            .first()
            .ok_or_else(|| AppError::Generic("攻略方法が選択されていません".to_string()))?,
        _ => {
            return Err(AppError::Generic(
                "予期しないコンポーネントタイプです".to_string(),
            ))
        }
    };

    let battle_style_id: i32 = battle_style_id_str
        .parse()
        .map_err(|_| AppError::Generic("攻略方法IDの解析に失敗しました".to_string()))?;

    // 攻略方法名をDBから取得
    let battle_style_repo = SeaOrmBattleStyleRepository::new();
    let battle_style = battle_style_repo
        .get_by_id(data.app_state.guild_db(), battle_style_id)
        .await?
        .ok_or_else(|| AppError::Generic("攻略方法が見つかりません".to_string()))?;

    info!(
        message_id = %message_id,
        battle_style_id = %battle_style_id,
        battle_style_name = %battle_style.display_name,
        "攻略方法が選択されました"
    );

    // Deferして処理時間を確保
    interaction.defer(&ctx.http).await?;

    // 対象メッセージを取得
    let channel_id = interaction.channel_id;
    let guild_id = interaction
        .guild_id
        .ok_or_else(|| AppError::Generic("ギルドIDが取得できません".to_string()))?
        .get();

    let target_message = channel_id
        .message(&ctx.http, message_id)
        .await
        .map_err(|e| {
            error!(error = %e, "メッセージの取得に失敗しました");
            AppError::Generic("対象のメッセージが見つかりませんでした".to_string())
        })?;

    // リファクタリングされたfacadeを呼び出す
    let result = crate::facades::recruitment::change::change_recruitment_information_internal(
        &data.app_state,
        &ctx.http,
        guild_id,
        &target_message,
        None,
        None,
        Some(battle_style_id),
    )
    .await;

    match result {
        Ok(_) => {
            interaction
                .edit_response(
                    &ctx.http,
                    poise::serenity_prelude::EditInteractionResponse::new()
                        .content(format!("攻略方法を「{}」に変更しました。", battle_style.display_name))
                        .components(vec![]),
                )
                .await?;
        }
        Err(e) => {
            error!(error = %e, "攻略方法変更に失敗しました");
            interaction
                .edit_response(
                    &ctx.http,
                    poise::serenity_prelude::EditInteractionResponse::new()
                        .content(format!("エラー: {}", e.user_message()))
                        .components(vec![]),
                )
                .await?;
        }
    }

    Ok(())
}

/// カスタムIDからメッセージIDを抽出
fn extract_message_id(custom_id: &str) -> Result<u64> {
    custom_id
        .split(':')
        .nth(1)
        .and_then(|s| s.parse::<u64>().ok())
        .ok_or_else(|| AppError::Generic("メッセージIDの抽出に失敗しました".to_string()))
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
    // 現在はスラッシュコマンド経由での変更を推奨
    Err(AppError::Generic(
        "日時変更機能は現在実装中です。スラッシュコマンド `/マルチバトル募集内容変更` をご利用ください。"
            .to_string(),
    ))
}
