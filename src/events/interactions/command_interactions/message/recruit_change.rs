use crate::types::{PoiseContext, Result};
use poise::serenity_prelude::{
    CreateActionRow, CreateInteractionResponse, CreateInteractionResponseMessage,
    CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption, Message,
};

/// メッセージコンテキストメニューから募集内容変更を開始
#[poise::command(context_menu_command = "募集内容変更")]
pub async fn recruit_change_context_menu(
    ctx: PoiseContext<'_>,
    message: Message,
) -> Result<()> {
    // カスタムIDにメッセージIDを含める
    let custom_id = format!("recruit_change_select_field:{}", message.id);

    // 変更する項目を選択するセレクトメニューを作成
    let select_menu = CreateSelectMenu::new(
        custom_id,
        CreateSelectMenuKind::String {
            options: vec![
                CreateSelectMenuOption::new("クエスト名", "quest")
                    .description("募集するクエストを変更します"),
                CreateSelectMenuOption::new("出発日時", "date")
                    .description("クエストの出発日時を変更します"),
                CreateSelectMenuOption::new("攻略方法", "battle_style")
                    .description("マルチバトルの攻略方法を変更します"),
            ],
        },
    )
    .placeholder("変更する項目を選択してください（複数選択可）")
    .min_values(1)
    .max_values(3);

    let components = vec![CreateActionRow::SelectMenu(select_menu)];

    // ApplicationContextの場合のみ応答可能
    match ctx {
        poise::Context::Application(app_ctx) => {
            app_ctx
                .interaction
                .create_response(
                    &ctx.serenity_context().http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content("変更する項目を選択してください")
                            .components(components)
                            .ephemeral(true),
                    ),
                )
                .await?;
        }
        _ => {
            return Err(crate::types::AppError::Generic(
                "このコマンドはコンテキストメニューからのみ使用できます".to_string(),
            ));
        }
    }

    Ok(())
}
