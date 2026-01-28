# Step 2: ドメイン用Value Objectの作成

## 目的

poise/serenityの型（`ChannelId`, `MessageId`, `GuildId`等）をビジネスロジック層で直接使用する代わりに、ドメイン固有のValue Objectを作成し、型の独立性を確保する。

## 概要

```
┌─────────────────────────────────────────────────────────────┐
│                 poise/serenity Types                         │
│    ChannelId, MessageId, GuildId, UserId, Message, etc.     │
└─────────────────────────────────────────────────────────────┘
                              ▲
                              │ 変換（Gateway層のみ）
                              │
┌─────────────────────────────────────────────────────────────┐
│                   Domain Value Objects                       │
│  DiscordChannelId, DiscordMessageId, MessageData, etc.      │
│         (Facade, Service, Repository で使用)                 │
└─────────────────────────────────────────────────────────────┘
```

## 作成するValue Object

### 1. 識別子型（ID Types）

```rust
// src/domain/types/discord_ids.rs

/// DiscordチャンネルID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DiscordChannelId(pub u64);

impl DiscordChannelId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn get(&self) -> u64 {
        self.0
    }
}

impl From<u64> for DiscordChannelId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

/// DiscordメッセージID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DiscordMessageId(pub u64);

impl DiscordMessageId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn get(&self) -> u64 {
        self.0
    }
}

/// DiscordギルドID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DiscordGuildId(pub u64);

/// DiscordユーザーID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DiscordUserId(pub u64);

/// DiscordロールID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DiscordRoleId(pub u64);

/// DiscordインタラクションID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DiscordInteractionId(pub u64);
```

### 2. メッセージ関連モデル

```rust
// src/domain/models/message.rs

use crate::domain::types::*;

/// メッセージコンテンツ（送信・編集用）
#[derive(Debug, Clone)]
pub struct MessageContent {
    /// テキストコンテンツ
    pub text: Option<String>,
    /// Embed一覧
    pub embeds: Vec<EmbedData>,
    /// UIコンポーネント一覧
    pub components: Vec<ComponentRow>,
}

impl MessageContent {
    /// テキストのみのメッセージを作成
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            text: Some(content.into()),
            embeds: Vec::new(),
            components: Vec::new(),
        }
    }

    /// Embedのみのメッセージを作成
    pub fn embed(embed: EmbedData) -> Self {
        Self {
            text: None,
            embeds: vec![embed],
            components: Vec::new(),
        }
    }

    /// コンポーネント付きメッセージを作成
    pub fn with_components(mut self, components: Vec<ComponentRow>) -> Self {
        self.components = components;
        self
    }
}

/// 取得したメッセージデータ
#[derive(Debug, Clone)]
pub struct MessageData {
    pub id: DiscordMessageId,
    pub channel_id: DiscordChannelId,
    pub author_id: DiscordUserId,
    pub content: String,
    pub embeds: Vec<EmbedData>,
    pub components: Vec<ComponentRow>,
    pub reactions: Vec<ReactionInfo>,
}
```

### 3. Embed関連モデル

```rust
// src/domain/models/embed.rs

/// Embedデータ
#[derive(Debug, Clone, Default)]
pub struct EmbedData {
    pub title: Option<String>,
    pub description: Option<String>,
    pub color: Option<u32>,
    pub fields: Vec<EmbedField>,
    pub footer: Option<EmbedFooter>,
    pub thumbnail_url: Option<String>,
    pub image_url: Option<String>,
    pub author: Option<EmbedAuthor>,
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

impl EmbedData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn color(mut self, color: u32) -> Self {
        self.color = Some(color);
        self
    }

    pub fn field(mut self, name: impl Into<String>, value: impl Into<String>, inline: bool) -> Self {
        self.fields.push(EmbedField {
            name: name.into(),
            value: value.into(),
            inline,
        });
        self
    }

    pub fn footer(mut self, text: impl Into<String>) -> Self {
        self.footer = Some(EmbedFooter {
            text: text.into(),
            icon_url: None,
        });
        self
    }
}

#[derive(Debug, Clone)]
pub struct EmbedField {
    pub name: String,
    pub value: String,
    pub inline: bool,
}

#[derive(Debug, Clone)]
pub struct EmbedFooter {
    pub text: String,
    pub icon_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EmbedAuthor {
    pub name: String,
    pub url: Option<String>,
    pub icon_url: Option<String>,
}
```

### 4. UIコンポーネント関連モデル

```rust
// src/domain/models/component.rs

/// コンポーネント行（ActionRow相当）
#[derive(Debug, Clone)]
pub struct ComponentRow {
    pub components: Vec<Component>,
}

impl ComponentRow {
    pub fn new(components: Vec<Component>) -> Self {
        Self { components }
    }

    pub fn buttons(buttons: Vec<ButtonData>) -> Self {
        Self {
            components: buttons.into_iter().map(Component::Button).collect(),
        }
    }

    pub fn select_menu(menu: SelectMenuData) -> Self {
        Self {
            components: vec![Component::SelectMenu(menu)],
        }
    }
}

/// コンポーネント
#[derive(Debug, Clone)]
pub enum Component {
    Button(ButtonData),
    SelectMenu(SelectMenuData),
}

/// ボタンデータ
#[derive(Debug, Clone)]
pub struct ButtonData {
    pub custom_id: String,
    pub label: String,
    pub style: ButtonStyle,
    pub disabled: bool,
    pub emoji: Option<String>,
}

impl ButtonData {
    pub fn primary(custom_id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            custom_id: custom_id.into(),
            label: label.into(),
            style: ButtonStyle::Primary,
            disabled: false,
            emoji: None,
        }
    }

    pub fn secondary(custom_id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            custom_id: custom_id.into(),
            label: label.into(),
            style: ButtonStyle::Secondary,
            disabled: false,
            emoji: None,
        }
    }

    pub fn danger(custom_id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            custom_id: custom_id.into(),
            label: label.into(),
            style: ButtonStyle::Danger,
            disabled: false,
            emoji: None,
        }
    }

    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }
}

/// ボタンスタイル
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonStyle {
    Primary,
    Secondary,
    Success,
    Danger,
    Link,
}

/// セレクトメニューデータ
#[derive(Debug, Clone)]
pub struct SelectMenuData {
    pub custom_id: String,
    pub placeholder: Option<String>,
    pub options: Vec<SelectOption>,
    pub min_values: Option<u8>,
    pub max_values: Option<u8>,
    pub disabled: bool,
}

#[derive(Debug, Clone)]
pub struct SelectOption {
    pub label: String,
    pub value: String,
    pub description: Option<String>,
    pub emoji: Option<String>,
    pub default: bool,
}
```

### 5. チャンネル関連モデル

```rust
// src/domain/models/channel.rs

use crate::domain::types::*;

/// チャンネル作成パラメータ
#[derive(Debug, Clone)]
pub struct ChannelCreateParams {
    pub name: String,
    pub channel_type: ChannelType,
    pub category_id: Option<DiscordChannelId>,
    pub topic: Option<String>,
    pub position: Option<u16>,
}

/// チャンネル編集パラメータ
#[derive(Debug, Clone, Default)]
pub struct ChannelEditParams {
    pub name: Option<String>,
    pub topic: Option<String>,
    pub position: Option<u16>,
}

/// チャンネルデータ
#[derive(Debug, Clone)]
pub struct ChannelData {
    pub id: DiscordChannelId,
    pub guild_id: Option<DiscordGuildId>,
    pub name: String,
    pub channel_type: ChannelType,
}

/// チャンネルタイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelType {
    Text,
    Voice,
    Category,
    News,
    Forum,
    Unknown,
}
```

### 6. リアクション関連モデル

```rust
// src/domain/models/reaction.rs

use crate::domain::types::*;

/// リアクション絵文字
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ReactionEmoji {
    /// Unicode絵文字
    Unicode(String),
    /// カスタム絵文字
    Custom {
        id: u64,
        name: String,
        animated: bool,
    },
}

impl ReactionEmoji {
    pub fn unicode(emoji: impl Into<String>) -> Self {
        Self::Unicode(emoji.into())
    }

    pub fn custom(id: u64, name: impl Into<String>) -> Self {
        Self::Custom {
            id,
            name: name.into(),
            animated: false,
        }
    }
}

/// リアクション情報
#[derive(Debug, Clone)]
pub struct ReactionInfo {
    pub emoji: ReactionEmoji,
    pub count: u64,
    pub me: bool,
}
```

### 7. ギルド関連モデル

```rust
// src/domain/models/guild.rs

use crate::domain::types::*;

/// ギルドメンバー
#[derive(Debug, Clone)]
pub struct GuildMember {
    pub user_id: DiscordUserId,
    pub nickname: Option<String>,
    pub roles: Vec<DiscordRoleId>,
    pub joined_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// ギルドロール
#[derive(Debug, Clone)]
pub struct GuildRole {
    pub id: DiscordRoleId,
    pub name: String,
    pub color: u32,
    pub position: i16,
    pub permissions: u64,
}

/// ギルド絵文字
#[derive(Debug, Clone)]
pub struct GuildEmoji {
    pub id: u64,
    pub name: String,
    pub animated: bool,
}
```

### 8. インタラクション関連モデル

```rust
// src/domain/models/interaction.rs

use crate::domain::types::*;
use crate::domain::models::message::MessageContent;

/// インタラクションデータ（受信用）
#[derive(Debug, Clone)]
pub struct InteractionData {
    pub id: DiscordInteractionId,
    pub guild_id: Option<DiscordGuildId>,
    pub channel_id: DiscordChannelId,
    pub user_id: DiscordUserId,
    pub custom_id: String,
    pub message_id: Option<DiscordMessageId>,
    pub values: Vec<String>,  // セレクトメニュー選択値
}

/// インタラクション応答
#[derive(Debug, Clone)]
pub struct InteractionResponse {
    pub content: MessageContent,
    pub ephemeral: bool,
}

impl InteractionResponse {
    pub fn new(content: MessageContent) -> Self {
        Self {
            content,
            ephemeral: false,
        }
    }

    pub fn ephemeral(mut self) -> Self {
        self.ephemeral = true;
        self
    }
}
```

### 9. オートコンプリート用モデル

```rust
// src/domain/models/autocomplete.rs

/// オートコンプリート選択肢
#[derive(Debug, Clone)]
pub struct AutocompleteOption {
    pub name: String,
    pub value: String,
}

impl AutocompleteOption {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

// 利便性のためのFrom実装
impl From<(String, String)> for AutocompleteOption {
    fn from((name, value): (String, String)) -> Self {
        Self { name, value }
    }
}

impl From<(&str, &str)> for AutocompleteOption {
    fn from((name, value): (&str, &str)) -> Self {
        Self {
            name: name.to_string(),
            value: value.to_string(),
        }
    }
}
```

## ディレクトリ構成

```
src/domain/
├── mod.rs
├── types/
│   ├── mod.rs
│   └── discord_ids.rs       # ID型定義
└── models/
    ├── mod.rs
    ├── message.rs           # メッセージ関連
    ├── embed.rs             # Embed関連
    ├── component.rs         # UIコンポーネント
    ├── channel.rs           # チャンネル関連
    ├── reaction.rs          # リアクション関連
    ├── guild.rs             # ギルド関連
    ├── interaction.rs       # インタラクション関連
    └── autocomplete.rs      # オートコンプリート
```

## 型変換（Gateway層で実装）

Gateway実装内でドメイン型⇔serenity型の変換を行う。

```rust
// src/gateway/impl/converters.rs

use poise::serenity_prelude as serenity;
use crate::domain::types::*;
use crate::domain::models::*;

// --- ID変換 ---

impl From<serenity::ChannelId> for DiscordChannelId {
    fn from(id: serenity::ChannelId) -> Self {
        Self(id.get())
    }
}

impl From<DiscordChannelId> for serenity::ChannelId {
    fn from(id: DiscordChannelId) -> Self {
        serenity::ChannelId::new(id.0)
    }
}

// MessageId, GuildId, UserId 等も同様

// --- Embed変換 ---

impl From<EmbedData> for serenity::CreateEmbed {
    fn from(embed: EmbedData) -> Self {
        let mut builder = serenity::CreateEmbed::new();

        if let Some(title) = embed.title {
            builder = builder.title(title);
        }
        if let Some(description) = embed.description {
            builder = builder.description(description);
        }
        if let Some(color) = embed.color {
            builder = builder.color(color);
        }
        for field in embed.fields {
            builder = builder.field(field.name, field.value, field.inline);
        }
        if let Some(footer) = embed.footer {
            builder = builder.footer(serenity::CreateEmbedFooter::new(footer.text));
        }

        builder
    }
}

// --- ボタンスタイル変換 ---

impl From<ButtonStyle> for serenity::ButtonStyle {
    fn from(style: ButtonStyle) -> Self {
        match style {
            ButtonStyle::Primary => serenity::ButtonStyle::Primary,
            ButtonStyle::Secondary => serenity::ButtonStyle::Secondary,
            ButtonStyle::Success => serenity::ButtonStyle::Success,
            ButtonStyle::Danger => serenity::ButtonStyle::Danger,
            ButtonStyle::Link => serenity::ButtonStyle::Link,
        }
    }
}

// --- コンポーネント変換 ---

impl From<ComponentRow> for serenity::CreateActionRow {
    fn from(row: ComponentRow) -> Self {
        let components: Vec<serenity::CreateActionRowComponent> = row
            .components
            .into_iter()
            .map(|c| match c {
                Component::Button(btn) => {
                    serenity::CreateActionRowComponent::Button(
                        serenity::CreateButton::new(btn.custom_id)
                            .label(btn.label)
                            .style(btn.style.into())
                            .disabled(btn.disabled)
                    )
                }
                Component::SelectMenu(menu) => {
                    // セレクトメニュー変換
                    // ...
                }
            })
            .collect();

        serenity::CreateActionRow::Buttons(components)
    }
}
```

## 移行パターン

### Before: serenity型を直接使用

```rust
use poise::serenity_prelude::{ChannelId, CreateEmbed, ButtonStyle};

pub fn create_recruitment_embed(channel_id: ChannelId) -> CreateEmbed {
    CreateEmbed::new()
        .title("募集")
        .description("参加者募集中")
        .color(0x00ff00)
}
```

### After: ドメイン型を使用

```rust
use crate::domain::types::DiscordChannelId;
use crate::domain::models::embed::EmbedData;

pub fn create_recruitment_embed(channel_id: DiscordChannelId) -> EmbedData {
    EmbedData::new()
        .title("募集")
        .description("参加者募集中")
        .color(0x00ff00)
}
```

## 完了条件

- [ ] すべてのID型が定義されている
- [ ] メッセージ関連モデルが定義されている
- [ ] Embed関連モデルが定義されている
- [ ] UIコンポーネントモデルが定義されている
- [ ] チャンネル関連モデルが定義されている
- [ ] リアクション関連モデルが定義されている
- [ ] ギルド関連モデルが定義されている
- [ ] インタラクション関連モデルが定義されている
- [ ] Gateway層で型変換が実装されている

## 注意事項

1. **Builderパターンを活用** - `EmbedData`等はメソッドチェーンで構築可能に
2. **Fromトレイトで相互変換** - Gateway層でのみ変換を行う
3. **既存のentityと混同しない** - これらはDiscord APIのValue Object
4. **必要最小限のフィールドから開始** - 使う機能だけを定義
