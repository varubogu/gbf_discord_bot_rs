//! ドメイン型からpoise/serenity型への変換ヘルパー
//!
//! Service/Facade層から返されるドメイン型を、Events層で
//! poise/serenity型に変換するためのヘルパー関数群。

use crate::types::discord::{
    ActionRowContent, AutocompleteOption, ButtonContent, ButtonStyleType, ComponentContent,
    EmbedContent, MessageContent, SelectMenuContent, SelectMenuKindContent,
};
use poise::serenity_prelude::{
    AutocompleteChoice, ButtonStyle, CreateActionRow, CreateButton, CreateEmbed,
    CreateEmbedAuthor, CreateEmbedFooter, CreateMessage, CreateSelectMenu, CreateSelectMenuKind,
    CreateSelectMenuOption, EditMessage, ReactionType,
};

/// AutocompleteOptionをAutocompleteChoiceに変換する
pub fn to_autocomplete_choice(option: AutocompleteOption) -> AutocompleteChoice {
    AutocompleteChoice::new(option.name, option.value)
}

/// Vec<AutocompleteOption>をVec<AutocompleteChoice>に変換する
pub fn to_autocomplete_choices(options: Vec<AutocompleteOption>) -> Vec<AutocompleteChoice> {
    options.into_iter().map(to_autocomplete_choice).collect()
}

/// EmbedContentをCreateEmbedに変換する
pub fn to_create_embed(embed: &EmbedContent) -> CreateEmbed {
    let mut create_embed = CreateEmbed::new();

    if let Some(title) = &embed.title {
        create_embed = create_embed.title(title);
    }

    if let Some(description) = &embed.description {
        create_embed = create_embed.description(description);
    }

    if let Some(color) = embed.color {
        create_embed = create_embed.color(color);
    }

    if let Some(footer) = &embed.footer {
        let mut footer_builder = CreateEmbedFooter::new(&footer.text);
        if let Some(icon_url) = &footer.icon_url {
            footer_builder = footer_builder.icon_url(icon_url);
        }
        create_embed = create_embed.footer(footer_builder);
    }

    if let Some(author) = &embed.author {
        let mut author_builder = CreateEmbedAuthor::new(&author.name);
        if let Some(url) = &author.url {
            author_builder = author_builder.url(url);
        }
        if let Some(icon_url) = &author.icon_url {
            author_builder = author_builder.icon_url(icon_url);
        }
        create_embed = create_embed.author(author_builder);
    }

    if let Some(thumbnail) = &embed.thumbnail_url {
        create_embed = create_embed.thumbnail(thumbnail);
    }

    if let Some(image) = &embed.image_url {
        create_embed = create_embed.image(image);
    }

    for field in &embed.fields {
        create_embed = create_embed.field(&field.name, &field.value, field.inline);
    }

    create_embed
}

/// ButtonStyleTypeをButtonStyleに変換する
fn to_button_style(style: &ButtonStyleType) -> ButtonStyle {
    match style {
        ButtonStyleType::Primary => ButtonStyle::Primary,
        ButtonStyleType::Secondary => ButtonStyle::Secondary,
        ButtonStyleType::Success => ButtonStyle::Success,
        ButtonStyleType::Danger => ButtonStyle::Danger,
        // Note: Linkスタイルは現在のserenityにないためSecondaryに変換
        ButtonStyleType::Link => ButtonStyle::Secondary,
    }
}

/// ButtonContentをCreateButtonに変換する
fn to_create_button(button: &ButtonContent) -> CreateButton {
    let mut create_button =
        CreateButton::new(&button.custom_id).style(to_button_style(&button.style));

    create_button = create_button.label(&button.label);

    if let Some(emoji) = &button.emoji {
        create_button = create_button.emoji(ReactionType::Unicode(emoji.clone()));
    }

    if button.disabled {
        create_button = create_button.disabled(true);
    }

    create_button
}

/// SelectMenuContentをCreateSelectMenuに変換する
fn to_create_select_menu(menu: &SelectMenuContent) -> CreateSelectMenu {
    let kind = match &menu.kind {
        SelectMenuKindContent::String { options } => {
            let serenity_options: Vec<CreateSelectMenuOption> = options
                .iter()
                .map(|opt| {
                    let mut option = CreateSelectMenuOption::new(&opt.label, &opt.value);
                    if let Some(desc) = &opt.description {
                        option = option.description(desc);
                    }
                    if let Some(emoji) = &opt.emoji {
                        option = option.emoji(ReactionType::Unicode(emoji.clone()));
                    }
                    if opt.default {
                        option = option.default_selection(true);
                    }
                    option
                })
                .collect();
            CreateSelectMenuKind::String {
                options: serenity_options,
            }
        }
        SelectMenuKindContent::User => CreateSelectMenuKind::User { default_users: None },
        SelectMenuKindContent::Role => CreateSelectMenuKind::Role { default_roles: None },
        SelectMenuKindContent::Channel => CreateSelectMenuKind::Channel {
            channel_types: None,
            default_channels: None,
        },
        SelectMenuKindContent::Mentionable => CreateSelectMenuKind::Mentionable {
            default_users: None,
            default_roles: None,
        },
    };

    let mut select_menu = CreateSelectMenu::new(&menu.custom_id, kind);

    if let Some(placeholder) = &menu.placeholder {
        select_menu = select_menu.placeholder(placeholder);
    }

    if let Some(min) = menu.min_values {
        select_menu = select_menu.min_values(min);
    }

    if let Some(max) = menu.max_values {
        select_menu = select_menu.max_values(max);
    }

    if menu.disabled {
        select_menu = select_menu.disabled(true);
    }

    select_menu
}

/// ActionRowContentをCreateActionRowに変換する
pub fn to_create_action_row(action_row: &ActionRowContent) -> CreateActionRow {
    if action_row.components.is_empty() {
        return CreateActionRow::Buttons(vec![]);
    }

    // 最初のコンポーネントで種別を判定
    match &action_row.components[0] {
        ComponentContent::Button(_) => {
            let buttons: Vec<CreateButton> = action_row
                .components
                .iter()
                .filter_map(|c| match c {
                    ComponentContent::Button(btn) => Some(to_create_button(btn)),
                    _ => None,
                })
                .collect();
            CreateActionRow::Buttons(buttons)
        }
        ComponentContent::SelectMenu(menu) => {
            CreateActionRow::SelectMenu(to_create_select_menu(menu))
        }
    }
}

/// MessageContentをCreateMessageに変換する
pub fn to_create_message(message: &MessageContent) -> CreateMessage {
    let mut create_message = CreateMessage::new();

    if let Some(text) = &message.text {
        create_message = create_message.content(text);
    }

    for embed in &message.embeds {
        create_message = create_message.embed(to_create_embed(embed));
    }

    let action_rows: Vec<CreateActionRow> =
        message.components.iter().map(to_create_action_row).collect();
    if !action_rows.is_empty() {
        create_message = create_message.components(action_rows);
    }

    create_message
}

/// MessageContentをEditMessageに変換する
pub fn to_edit_message(message: &MessageContent) -> EditMessage {
    let mut edit_message = EditMessage::new();

    if let Some(text) = &message.text {
        edit_message = edit_message.content(text);
    }

    for embed in &message.embeds {
        edit_message = edit_message.embed(to_create_embed(embed));
    }

    let action_rows: Vec<CreateActionRow> =
        message.components.iter().map(to_create_action_row).collect();
    if !action_rows.is_empty() {
        edit_message = edit_message.components(action_rows);
    }

    edit_message
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_autocomplete_choice() {
        let option = AutocompleteOption::new("テスト", "value");
        // AutocompleteChoice の内部フィールドにはアクセスできないため、
        // コンパイルと変換処理がエラーなく完了することのみを確認する
        let _choice: AutocompleteChoice = to_autocomplete_choice(option);
    }

    #[test]
    fn test_to_autocomplete_choices() {
        let options = vec![
            AutocompleteOption::new("オプション1", "1"),
            AutocompleteOption::new("オプション2", "2"),
        ];
        let choices = to_autocomplete_choices(options);
        assert_eq!(choices.len(), 2);
    }

    #[test]
    fn test_to_create_embed_basic() {
        let embed = EmbedContent::new()
            .with_title("テストタイトル")
            .with_description("テスト説明")
            .with_color(0x00ff00);

        let create_embed = to_create_embed(&embed);
        // CreateEmbedのフィールドは直接アクセスできないが、エラーなく変換できることを確認
        let _ = create_embed;
    }

    #[test]
    fn test_to_create_embed_with_fields() {
        let embed = EmbedContent::new()
            .with_title("タイトル")
            .with_field("フィールド1", "値1", true)
            .with_field("フィールド2", "値2", false);

        let create_embed = to_create_embed(&embed);
        let _ = create_embed;
    }

    #[test]
    fn test_to_create_message() {
        let message = MessageContent::new()
            .with_text("テストメッセージ")
            .with_embed(
                EmbedContent::new()
                    .with_title("タイトル")
                    .with_description("説明"),
            );

        let create_message = to_create_message(&message);
        let _ = create_message;
    }
}
