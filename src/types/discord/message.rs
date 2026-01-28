//! メッセージ関連のドメイン型
//!
//! Discordメッセージの作成・編集に必要なデータを表現する型を定義する。
//! poise/serenityのCreateMessage, EditMessage等の代替となる。

use super::{DiscordChannelId, DiscordMessageId, DiscordUserId};

/// メッセージ送信用のコンテンツ
#[derive(Debug, Clone, Default)]
pub struct MessageContent {
    /// テキストコンテンツ
    pub text: Option<String>,
    /// Embed一覧
    pub embeds: Vec<EmbedContent>,
    /// アクションロー（ボタン、セレクトメニュー等）
    pub components: Vec<ActionRowContent>,
}

impl MessageContent {
    /// 新しいMessageContentを作成する
    pub fn new() -> Self {
        Self::default()
    }

    /// テキストのみのメッセージを作成する
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            text: Some(text.into()),
            ..Default::default()
        }
    }

    /// テキストを設定する
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Embedを追加する
    pub fn with_embed(mut self, embed: EmbedContent) -> Self {
        self.embeds.push(embed);
        self
    }

    /// 複数のEmbedを追加する
    pub fn with_embeds(mut self, embeds: Vec<EmbedContent>) -> Self {
        self.embeds.extend(embeds);
        self
    }

    /// アクションローを追加する
    pub fn with_component(mut self, component: ActionRowContent) -> Self {
        self.components.push(component);
        self
    }

    /// 複数のアクションローを追加する
    pub fn with_components(mut self, components: Vec<ActionRowContent>) -> Self {
        self.components.extend(components);
        self
    }
}

/// 取得したメッセージデータ
#[derive(Debug, Clone)]
pub struct MessageData {
    /// メッセージID
    pub id: DiscordMessageId,
    /// チャンネルID
    pub channel_id: DiscordChannelId,
    /// 送信者ID
    pub author_id: DiscordUserId,
    /// テキストコンテンツ
    pub content: String,
    /// Embed一覧
    pub embeds: Vec<EmbedData>,
    /// コンポーネント一覧
    pub components: Vec<ActionRowData>,
    /// ピン留めされているか
    pub pinned: bool,
}

/// Embedコンテンツ（送信用）
#[derive(Debug, Clone, Default)]
pub struct EmbedContent {
    /// タイトル
    pub title: Option<String>,
    /// 説明
    pub description: Option<String>,
    /// カラーコード（RGB）
    pub color: Option<u32>,
    /// フィールド一覧
    pub fields: Vec<EmbedFieldContent>,
    /// フッター
    pub footer: Option<EmbedFooterContent>,
    /// サムネイルURL
    pub thumbnail_url: Option<String>,
    /// 画像URL
    pub image_url: Option<String>,
    /// 作成者
    pub author: Option<EmbedAuthorContent>,
    /// タイムスタンプ（ISO 8601形式）
    pub timestamp: Option<String>,
}

impl EmbedContent {
    /// 新しいEmbedContentを作成する
    pub fn new() -> Self {
        Self::default()
    }

    /// タイトルを設定する
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// 説明を設定する
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// カラーを設定する
    pub fn with_color(mut self, color: u32) -> Self {
        self.color = Some(color);
        self
    }

    /// フィールドを追加する
    pub fn with_field(
        mut self,
        name: impl Into<String>,
        value: impl Into<String>,
        inline: bool,
    ) -> Self {
        self.fields.push(EmbedFieldContent {
            name: name.into(),
            value: value.into(),
            inline,
        });
        self
    }

    /// フッターを設定する
    pub fn with_footer(mut self, text: impl Into<String>) -> Self {
        self.footer = Some(EmbedFooterContent {
            text: text.into(),
            icon_url: None,
        });
        self
    }

    /// アイコン付きフッターを設定する
    pub fn with_footer_and_icon(
        mut self,
        text: impl Into<String>,
        icon_url: impl Into<String>,
    ) -> Self {
        self.footer = Some(EmbedFooterContent {
            text: text.into(),
            icon_url: Some(icon_url.into()),
        });
        self
    }

    /// サムネイルを設定する
    pub fn with_thumbnail(mut self, url: impl Into<String>) -> Self {
        self.thumbnail_url = Some(url.into());
        self
    }

    /// 画像を設定する
    pub fn with_image(mut self, url: impl Into<String>) -> Self {
        self.image_url = Some(url.into());
        self
    }

    /// 作成者を設定する
    pub fn with_author(mut self, name: impl Into<String>) -> Self {
        self.author = Some(EmbedAuthorContent {
            name: name.into(),
            url: None,
            icon_url: None,
        });
        self
    }

    /// タイムスタンプを設定する
    pub fn with_timestamp(mut self, timestamp: impl Into<String>) -> Self {
        self.timestamp = Some(timestamp.into());
        self
    }
}

/// Embedフィールドコンテンツ
#[derive(Debug, Clone)]
pub struct EmbedFieldContent {
    /// フィールド名
    pub name: String,
    /// フィールド値
    pub value: String,
    /// インラインで表示するか
    pub inline: bool,
}

/// Embedフッターコンテンツ
#[derive(Debug, Clone)]
pub struct EmbedFooterContent {
    /// テキスト
    pub text: String,
    /// アイコンURL
    pub icon_url: Option<String>,
}

/// Embed作成者コンテンツ
#[derive(Debug, Clone)]
pub struct EmbedAuthorContent {
    /// 名前
    pub name: String,
    /// URL
    pub url: Option<String>,
    /// アイコンURL
    pub icon_url: Option<String>,
}

/// 取得したEmbedデータ
#[derive(Debug, Clone)]
pub struct EmbedData {
    /// タイトル
    pub title: Option<String>,
    /// 説明
    pub description: Option<String>,
    /// カラーコード
    pub color: Option<u32>,
    /// フィールド一覧
    pub fields: Vec<EmbedFieldData>,
    /// フッターテキスト
    pub footer_text: Option<String>,
}

/// 取得したEmbedフィールドデータ
#[derive(Debug, Clone)]
pub struct EmbedFieldData {
    /// フィールド名
    pub name: String,
    /// フィールド値
    pub value: String,
    /// インラインか
    pub inline: bool,
}

/// アクションローコンテンツ（送信用）
#[derive(Debug, Clone)]
pub struct ActionRowContent {
    /// コンポーネント一覧
    pub components: Vec<ComponentContent>,
}

impl ActionRowContent {
    /// ボタン一覧からアクションローを作成する
    pub fn buttons(buttons: Vec<ButtonContent>) -> Self {
        Self {
            components: buttons.into_iter().map(ComponentContent::Button).collect(),
        }
    }

    /// セレクトメニューからアクションローを作成する
    pub fn select_menu(menu: SelectMenuContent) -> Self {
        Self {
            components: vec![ComponentContent::SelectMenu(menu)],
        }
    }
}

/// コンポーネントコンテンツ
#[derive(Debug, Clone)]
pub enum ComponentContent {
    /// ボタン
    Button(ButtonContent),
    /// セレクトメニュー
    SelectMenu(SelectMenuContent),
}

/// ボタンコンテンツ
#[derive(Debug, Clone)]
pub struct ButtonContent {
    /// カスタムID
    pub custom_id: String,
    /// ラベル
    pub label: String,
    /// スタイル
    pub style: ButtonStyleType,
    /// 無効化されているか
    pub disabled: bool,
    /// 絵文字
    pub emoji: Option<String>,
}

impl ButtonContent {
    /// 新しいボタンを作成する
    pub fn new(custom_id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            custom_id: custom_id.into(),
            label: label.into(),
            style: ButtonStyleType::Primary,
            disabled: false,
            emoji: None,
        }
    }

    /// スタイルを設定する
    pub fn with_style(mut self, style: ButtonStyleType) -> Self {
        self.style = style;
        self
    }

    /// 無効化状態を設定する
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 絵文字を設定する
    pub fn with_emoji(mut self, emoji: impl Into<String>) -> Self {
        self.emoji = Some(emoji.into());
        self
    }
}

/// ボタンスタイル
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonStyleType {
    /// プライマリ（青）
    Primary,
    /// セカンダリ（グレー）
    Secondary,
    /// 成功（緑）
    Success,
    /// 危険（赤）
    Danger,
    /// リンク
    Link,
}

/// セレクトメニューコンテンツ
#[derive(Debug, Clone)]
pub struct SelectMenuContent {
    /// カスタムID
    pub custom_id: String,
    /// プレースホルダーテキスト
    pub placeholder: Option<String>,
    /// 最小選択数
    pub min_values: Option<u8>,
    /// 最大選択数
    pub max_values: Option<u8>,
    /// 無効化されているか
    pub disabled: bool,
    /// メニュー種別
    pub kind: SelectMenuKindContent,
}

impl SelectMenuContent {
    /// 文字列選択メニューを作成する
    pub fn string_select(
        custom_id: impl Into<String>,
        options: Vec<SelectMenuOptionContent>,
    ) -> Self {
        Self {
            custom_id: custom_id.into(),
            placeholder: None,
            min_values: None,
            max_values: None,
            disabled: false,
            kind: SelectMenuKindContent::String { options },
        }
    }

    /// ユーザー選択メニューを作成する
    pub fn user_select(custom_id: impl Into<String>) -> Self {
        Self {
            custom_id: custom_id.into(),
            placeholder: None,
            min_values: None,
            max_values: None,
            disabled: false,
            kind: SelectMenuKindContent::User,
        }
    }

    /// チャンネル選択メニューを作成する
    pub fn channel_select(custom_id: impl Into<String>) -> Self {
        Self {
            custom_id: custom_id.into(),
            placeholder: None,
            min_values: None,
            max_values: None,
            disabled: false,
            kind: SelectMenuKindContent::Channel,
        }
    }

    /// プレースホルダーを設定する
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// 最小選択数を設定する
    pub fn with_min_values(mut self, min: u8) -> Self {
        self.min_values = Some(min);
        self
    }

    /// 最大選択数を設定する
    pub fn with_max_values(mut self, max: u8) -> Self {
        self.max_values = Some(max);
        self
    }
}

/// セレクトメニュー種別
#[derive(Debug, Clone)]
pub enum SelectMenuKindContent {
    /// 文字列選択
    String {
        options: Vec<SelectMenuOptionContent>,
    },
    /// ユーザー選択
    User,
    /// ロール選択
    Role,
    /// チャンネル選択
    Channel,
    /// メンション可能選択
    Mentionable,
}

/// セレクトメニューオプションコンテンツ
#[derive(Debug, Clone)]
pub struct SelectMenuOptionContent {
    /// ラベル
    pub label: String,
    /// 値
    pub value: String,
    /// 説明
    pub description: Option<String>,
    /// 絵文字
    pub emoji: Option<String>,
    /// デフォルト選択か
    pub default: bool,
}

impl SelectMenuOptionContent {
    /// 新しいオプションを作成する
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            description: None,
            emoji: None,
            default: false,
        }
    }

    /// 説明を設定する
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// 絵文字を設定する
    pub fn with_emoji(mut self, emoji: impl Into<String>) -> Self {
        self.emoji = Some(emoji.into());
        self
    }

    /// デフォルト選択に設定する
    pub fn with_default(mut self, default: bool) -> Self {
        self.default = default;
        self
    }
}

/// 取得したアクションローデータ
#[derive(Debug, Clone)]
pub struct ActionRowData {
    /// コンポーネント一覧
    pub components: Vec<ComponentData>,
}

/// 取得したコンポーネントデータ
#[derive(Debug, Clone)]
pub enum ComponentData {
    /// ボタン
    Button(ButtonData),
    /// セレクトメニュー
    SelectMenu(SelectMenuData),
    /// 不明なコンポーネント
    Unknown,
}

/// 取得したボタンデータ
#[derive(Debug, Clone)]
pub struct ButtonData {
    /// カスタムID
    pub custom_id: Option<String>,
    /// ラベル
    pub label: Option<String>,
    /// 無効化されているか
    pub disabled: bool,
}

/// 取得したセレクトメニューデータ
#[derive(Debug, Clone)]
pub struct SelectMenuData {
    /// カスタムID
    pub custom_id: String,
    /// プレースホルダー
    pub placeholder: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_content_builder() {
        let content = MessageContent::new().with_text("Hello, World!").with_embed(
            EmbedContent::new()
                .with_title("Test Embed")
                .with_description("This is a test")
                .with_color(0xFF0000),
        );

        assert_eq!(content.text, Some("Hello, World!".to_string()));
        assert_eq!(content.embeds.len(), 1);
        assert_eq!(content.embeds[0].title, Some("Test Embed".to_string()));
    }

    #[test]
    fn test_button_content_builder() {
        let button = ButtonContent::new("btn_test", "Click Me")
            .with_style(ButtonStyleType::Success)
            .with_disabled(false);

        assert_eq!(button.custom_id, "btn_test");
        assert_eq!(button.label, "Click Me");
        assert_eq!(button.style, ButtonStyleType::Success);
    }

    #[test]
    fn test_select_menu_builder() {
        let menu = SelectMenuContent::string_select(
            "menu_test",
            vec![
                SelectMenuOptionContent::new("Option 1", "opt1"),
                SelectMenuOptionContent::new("Option 2", "opt2").with_description("Second option"),
            ],
        )
        .with_placeholder("Select an option");

        assert_eq!(menu.custom_id, "menu_test");
        assert_eq!(menu.placeholder, Some("Select an option".to_string()));
    }
}
