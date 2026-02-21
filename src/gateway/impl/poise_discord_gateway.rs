//! Poise/Serenityを使用したDiscord Gateway実装
//!
//! 本番環境で使用するDiscord Gateway実装。
//! poise/serenityのHTTPクライアントを使用してDiscord APIを呼び出す。

use std::sync::Arc;

use async_trait::async_trait;
use poise::serenity_prelude::{
    ChannelId, ChannelType, CreateChannel, CreateMessage, EditChannel, EditMessage, GetMessages,
    GuildId, Http, MessageId, ReactionType, UserId,
};

use crate::errors::GatewayError;
use crate::gateway::{
    DiscordChannelGateway, DiscordGuildGateway, DiscordInteractionGateway, DiscordMessageGateway,
    DiscordReactionGateway,
};
use crate::types::discord::{
    ActionRowContent, ActionRowData, ButtonContent, ButtonData, ButtonStyleType,
    ChannelCreateParams, ChannelData, ChannelEditParams, ChannelKind, ComponentContent,
    ComponentData, DiscordChannelId, DiscordEmojiId, DiscordGuildId, DiscordInteractionId,
    DiscordMessageId, DiscordRoleId, DiscordUserId, EmbedContent, EmbedData, EmbedFieldData,
    GuildEmoji, GuildMember, GuildRole, InteractionResponse, MessageContent, MessageData,
    ReactionData, ReactionEmoji, SelectMenuContent, SelectMenuData, SelectMenuKindContent,
};

fn serenity_message_id_opt(message_id: DiscordMessageId) -> Option<MessageId> {
    let id = message_id.get();
    if id == 0 {
        return None;
    }
    Some(MessageId::new(id))
}

fn serenity_message_id(message_id: DiscordMessageId) -> Result<MessageId, GatewayError> {
    serenity_message_id_opt(message_id).ok_or_else(|| {
        // 0 はDiscordのSnowflakeとして不正であり、上位層の状態不整合を示す
        GatewayError::internal("DiscordメッセージIDが不正です（0）")
    })
}

/// Poise/Serenityを使用したDiscord Gateway実装
#[derive(Clone)]
pub struct PoiseDiscordGateway {
    /// HTTPクライアント
    http: Arc<Http>,
}

impl PoiseDiscordGateway {
    /// 新しいPoiseDiscordGatewayを作成する
    pub fn new(http: Arc<Http>) -> Self {
        Self { http }
    }

    /// HTTPクライアントを取得する
    pub fn http(&self) -> &Arc<Http> {
        &self.http
    }
}

// === MessageContent -> CreateMessage 変換 ===

impl MessageContent {
    /// SerenityのCreateMessageに変換する
    pub fn into_serenity_message(self) -> CreateMessage {
        let mut message = CreateMessage::new();

        if let Some(text) = self.text {
            message = message.content(text);
        }

        for embed in self.embeds {
            message = message.embed(embed.into_serenity_embed());
        }

        for component in self.components {
            message = message.components(vec![component.into_serenity_action_row()]);
        }

        message
    }

    /// SerenityのEditMessageに変換する
    pub fn into_serenity_edit_message(self) -> EditMessage {
        let mut message = EditMessage::new();

        if let Some(text) = self.text {
            message = message.content(text);
        }

        for embed in self.embeds {
            message = message.embed(embed.into_serenity_embed());
        }

        for component in self.components {
            message = message.components(vec![component.into_serenity_action_row()]);
        }

        message
    }
}

impl EmbedContent {
    /// SerenityのCreateEmbedに変換する
    fn into_serenity_embed(self) -> poise::serenity_prelude::CreateEmbed {
        let mut embed = poise::serenity_prelude::CreateEmbed::new();

        if let Some(title) = self.title {
            embed = embed.title(title);
        }

        if let Some(description) = self.description {
            embed = embed.description(description);
        }

        if let Some(color) = self.color {
            embed = embed.color(color);
        }

        for field in self.fields {
            embed = embed.field(field.name, field.value, field.inline);
        }

        if let Some(footer) = self.footer {
            let mut footer_builder = poise::serenity_prelude::CreateEmbedFooter::new(footer.text);
            if let Some(icon_url) = footer.icon_url {
                footer_builder = footer_builder.icon_url(icon_url);
            }
            embed = embed.footer(footer_builder);
        }

        if let Some(thumbnail_url) = self.thumbnail_url {
            embed = embed.thumbnail(thumbnail_url);
        }

        if let Some(image_url) = self.image_url {
            embed = embed.image(image_url);
        }

        if let Some(author) = self.author {
            let mut author_builder = poise::serenity_prelude::CreateEmbedAuthor::new(author.name);
            if let Some(url) = author.url {
                author_builder = author_builder.url(url);
            }
            if let Some(icon_url) = author.icon_url {
                author_builder = author_builder.icon_url(icon_url);
            }
            embed = embed.author(author_builder);
        }

        if let Some(timestamp) = self.timestamp
            && let Ok(ts) = timestamp.parse::<poise::serenity_prelude::Timestamp>()
        {
            embed = embed.timestamp(ts);
        }

        embed
    }
}

impl ActionRowContent {
    /// SerenityのCreateActionRowに変換する
    fn into_serenity_action_row(self) -> poise::serenity_prelude::CreateActionRow {
        let components: Vec<_> = self
            .components
            .into_iter()
            .map(|c| match c {
                ComponentContent::Button(btn) => {
                    poise::serenity_prelude::CreateActionRow::Buttons(vec![
                        btn.into_serenity_button(),
                    ])
                }
                ComponentContent::SelectMenu(menu) => {
                    poise::serenity_prelude::CreateActionRow::SelectMenu(
                        menu.into_serenity_select_menu(),
                    )
                }
            })
            .collect();

        // 全てのコンポーネントが同じ種類であることを想定
        components
            .into_iter()
            .next()
            .unwrap_or_else(|| poise::serenity_prelude::CreateActionRow::Buttons(vec![]))
    }
}

impl ButtonContent {
    /// SerenityのCreateButtonに変換する
    ///
    /// 注意: ButtonStyleType::Linkは現在のserenityバージョンでは
    /// URLリンクボタン専用であり、custom_idを持つ通常ボタンとは異なる。
    /// ここではLinkスタイルは暫定的にSecondaryとして扱う。
    fn into_serenity_button(self) -> poise::serenity_prelude::CreateButton {
        let style = match self.style {
            ButtonStyleType::Primary => poise::serenity_prelude::ButtonStyle::Primary,
            ButtonStyleType::Secondary => poise::serenity_prelude::ButtonStyle::Secondary,
            ButtonStyleType::Success => poise::serenity_prelude::ButtonStyle::Success,
            ButtonStyleType::Danger => poise::serenity_prelude::ButtonStyle::Danger,
            // Linkスタイルはcustom_idではなくURLを必要とするため、
            // 暫定的にSecondaryとして扱う
            ButtonStyleType::Link => poise::serenity_prelude::ButtonStyle::Secondary,
        };

        let mut button = poise::serenity_prelude::CreateButton::new(self.custom_id)
            .label(self.label)
            .style(style)
            .disabled(self.disabled);

        if let Some(emoji) = self.emoji {
            button = button.emoji(ReactionType::Unicode(emoji));
        }

        button
    }
}

impl SelectMenuContent {
    /// SerenityのCreateSelectMenuに変換する
    fn into_serenity_select_menu(self) -> poise::serenity_prelude::CreateSelectMenu {
        use poise::serenity_prelude::{
            CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption,
        };

        let kind = match self.kind {
            SelectMenuKindContent::String { options } => {
                let serenity_options: Vec<_> = options
                    .into_iter()
                    .map(|opt| {
                        let mut menu_opt = CreateSelectMenuOption::new(opt.label, opt.value);
                        if let Some(desc) = opt.description {
                            menu_opt = menu_opt.description(desc);
                        }
                        if let Some(emoji) = opt.emoji {
                            menu_opt = menu_opt.emoji(ReactionType::Unicode(emoji));
                        }
                        if opt.default {
                            menu_opt = menu_opt.default_selection(true);
                        }
                        menu_opt
                    })
                    .collect();
                CreateSelectMenuKind::String {
                    options: serenity_options,
                }
            }
            SelectMenuKindContent::User => CreateSelectMenuKind::User {
                default_users: None,
            },
            SelectMenuKindContent::Role => CreateSelectMenuKind::Role {
                default_roles: None,
            },
            SelectMenuKindContent::Channel => CreateSelectMenuKind::Channel {
                default_channels: None,
                channel_types: None,
            },
            SelectMenuKindContent::Mentionable => CreateSelectMenuKind::Mentionable {
                default_users: None,
                default_roles: None,
            },
        };

        let mut menu = CreateSelectMenu::new(self.custom_id, kind).disabled(self.disabled);

        if let Some(placeholder) = self.placeholder {
            menu = menu.placeholder(placeholder);
        }

        if let Some(min) = self.min_values {
            menu = menu.min_values(min);
        }

        if let Some(max) = self.max_values {
            menu = menu.max_values(max);
        }

        menu
    }
}

// === Serenity -> ドメイン型 変換 ===

impl From<poise::serenity_prelude::Message> for MessageData {
    fn from(msg: poise::serenity_prelude::Message) -> Self {
        Self {
            id: DiscordMessageId::new(msg.id.get()),
            channel_id: DiscordChannelId::new(msg.channel_id.get()),
            author_id: DiscordUserId::new(msg.author.id.get()),
            content: msg.content,
            embeds: msg.embeds.into_iter().map(EmbedData::from).collect(),
            components: msg
                .components
                .into_iter()
                .map(ActionRowData::from)
                .collect(),
            reactions: msg.reactions.into_iter().map(ReactionData::from).collect(),
            pinned: msg.pinned,
            referenced_message_id: msg
                .referenced_message
                .as_ref()
                .map(|m| DiscordMessageId::new(m.id.get())),
        }
    }
}

impl From<poise::serenity_prelude::MessageReaction> for ReactionData {
    fn from(reaction: poise::serenity_prelude::MessageReaction) -> Self {
        Self {
            emoji: ReactionEmoji::from(reaction.reaction_type),
            count: reaction.count,
        }
    }
}

impl From<ReactionType> for ReactionEmoji {
    fn from(rt: ReactionType) -> Self {
        match rt {
            ReactionType::Unicode(s) => ReactionEmoji::Unicode(s),
            ReactionType::Custom { animated, id, name } => ReactionEmoji::Custom {
                id: DiscordEmojiId::new(id.get()),
                name: name.unwrap_or_default(),
                animated,
            },
            _ => ReactionEmoji::Unicode("❓".to_string()),
        }
    }
}

impl From<poise::serenity_prelude::Embed> for EmbedData {
    fn from(embed: poise::serenity_prelude::Embed) -> Self {
        Self {
            title: embed.title,
            description: embed.description,
            color: embed.colour.map(|c| c.0),
            fields: embed.fields.into_iter().map(EmbedFieldData::from).collect(),
            footer_text: embed.footer.map(|f| f.text),
        }
    }
}

impl From<poise::serenity_prelude::EmbedField> for EmbedFieldData {
    fn from(field: poise::serenity_prelude::EmbedField) -> Self {
        Self {
            name: field.name,
            value: field.value,
            inline: field.inline,
        }
    }
}

impl From<poise::serenity_prelude::ActionRow> for ActionRowData {
    fn from(row: poise::serenity_prelude::ActionRow) -> Self {
        Self {
            components: row
                .components
                .into_iter()
                .map(ComponentData::from)
                .collect(),
        }
    }
}

impl From<poise::serenity_prelude::ActionRowComponent> for ComponentData {
    fn from(component: poise::serenity_prelude::ActionRowComponent) -> Self {
        match component {
            poise::serenity_prelude::ActionRowComponent::Button(btn) => {
                // serenity 0.12+ではButtonの構造が変更されている
                // custom_idはButtonKind enum内に格納されている
                // NonLinkバリアントの場合のみcustom_idを持つ
                let custom_id = match &btn.data {
                    poise::serenity_prelude::ButtonKind::NonLink { custom_id, .. } => {
                        Some(custom_id.clone())
                    }
                    poise::serenity_prelude::ButtonKind::Link { .. }
                    | poise::serenity_prelude::ButtonKind::Premium { .. } => None,
                };
                ComponentData::Button(ButtonData {
                    custom_id,
                    label: btn.label.clone(),
                    disabled: btn.disabled,
                })
            }
            poise::serenity_prelude::ActionRowComponent::SelectMenu(menu) => {
                ComponentData::SelectMenu(SelectMenuData {
                    custom_id: menu.custom_id.clone().unwrap_or_default(),
                    placeholder: menu.placeholder.clone(),
                })
            }
            _ => ComponentData::Unknown,
        }
    }
}

// === ReactionEmoji -> ReactionType 変換 ===

impl From<ReactionEmoji> for ReactionType {
    fn from(emoji: ReactionEmoji) -> Self {
        match emoji {
            ReactionEmoji::Unicode(s) => ReactionType::Unicode(s),
            ReactionEmoji::Custom { id, name, animated } => ReactionType::Custom {
                animated,
                id: poise::serenity_prelude::EmojiId::new(id.get()),
                name: Some(name),
            },
        }
    }
}

// === ChannelKind 変換 ===

impl From<ChannelType> for ChannelKind {
    fn from(ct: ChannelType) -> Self {
        match ct {
            ChannelType::Text => ChannelKind::Text,
            ChannelType::Voice => ChannelKind::Voice,
            ChannelType::Category => ChannelKind::Category,
            ChannelType::News => ChannelKind::Announcement,
            ChannelType::PublicThread => ChannelKind::PublicThread,
            ChannelType::PrivateThread => ChannelKind::PrivateThread,
            ChannelType::Stage => ChannelKind::Stage,
            ChannelType::Forum => ChannelKind::Forum,
            _ => ChannelKind::Unknown,
        }
    }
}

impl From<ChannelKind> for ChannelType {
    fn from(kind: ChannelKind) -> Self {
        match kind {
            ChannelKind::Text => ChannelType::Text,
            ChannelKind::Voice => ChannelType::Voice,
            ChannelKind::Category => ChannelType::Category,
            ChannelKind::Announcement => ChannelType::News,
            ChannelKind::PublicThread => ChannelType::PublicThread,
            ChannelKind::PrivateThread => ChannelType::PrivateThread,
            ChannelKind::Stage => ChannelType::Stage,
            ChannelKind::Forum => ChannelType::Forum,
            ChannelKind::Unknown => ChannelType::Text,
        }
    }
}

// === Gateway トレイト実装 ===

#[async_trait]
impl DiscordMessageGateway for PoiseDiscordGateway {
    async fn send_message(
        &self,
        channel_id: DiscordChannelId,
        content: MessageContent,
    ) -> Result<DiscordMessageId, GatewayError> {
        let serenity_channel_id = ChannelId::new(channel_id.get());
        let create_message = content.into_serenity_message();

        let message = serenity_channel_id
            .send_message(&self.http, create_message)
            .await
            .map_err(GatewayError::send_message_failed)?;

        Ok(DiscordMessageId::new(message.id.get()))
    }

    async fn edit_message(
        &self,
        channel_id: DiscordChannelId,
        message_id: DiscordMessageId,
        content: MessageContent,
    ) -> Result<(), GatewayError> {
        let serenity_channel_id = ChannelId::new(channel_id.get());
        let serenity_message_id = serenity_message_id(message_id)?;
        let edit_message = content.into_serenity_edit_message();

        serenity_channel_id
            .edit_message(&self.http, serenity_message_id, edit_message)
            .await
            .map_err(GatewayError::edit_message_failed)?;

        Ok(())
    }

    async fn delete_message(
        &self,
        channel_id: DiscordChannelId,
        message_id: DiscordMessageId,
    ) -> Result<(), GatewayError> {
        let serenity_channel_id = ChannelId::new(channel_id.get());
        let serenity_message_id = serenity_message_id(message_id)?;

        serenity_channel_id
            .delete_message(&self.http, serenity_message_id)
            .await
            .map_err(GatewayError::delete_message_failed)?;

        Ok(())
    }

    async fn get_message(
        &self,
        channel_id: DiscordChannelId,
        message_id: DiscordMessageId,
    ) -> Result<MessageData, GatewayError> {
        let serenity_channel_id = ChannelId::new(channel_id.get());
        let serenity_message_id = serenity_message_id(message_id)?;

        let message = serenity_channel_id
            .message(&self.http, serenity_message_id)
            .await
            .map_err(GatewayError::get_message_failed)?;

        Ok(MessageData::from(message))
    }

    async fn get_messages(
        &self,
        channel_id: DiscordChannelId,
        limit: u8,
    ) -> Result<Vec<MessageData>, GatewayError> {
        let serenity_channel_id = ChannelId::new(channel_id.get());

        let messages = serenity_channel_id
            .messages(&self.http, GetMessages::new().limit(limit))
            .await
            .map_err(GatewayError::get_message_failed)?;

        Ok(messages.into_iter().map(MessageData::from).collect())
    }

    async fn send_reply(
        &self,
        channel_id: DiscordChannelId,
        reply_to_message_id: DiscordMessageId,
        content: MessageContent,
        fallback_context: Option<String>,
    ) -> Result<DiscordMessageId, GatewayError> {
        use poise::serenity_prelude::MessageReference;

        let serenity_channel_id = ChannelId::new(channel_id.get());
        let Some(serenity_message_id) = serenity_message_id_opt(reply_to_message_id) else {
            tracing::warn!(
                channel_id = %channel_id,
                reply_to = %reply_to_message_id,
                "返信先メッセージIDが未設定（0）のため、通常メッセージとして送信します"
            );

            // 文脈情報を付加して通常メッセージとして送信
            let fallback_content = if let Some(context) = fallback_context {
                let original_text = content.text.clone().unwrap_or_default();
                MessageContent::text(format!("【{context}】\n{original_text}"))
            } else {
                content
            };

            return self.send_message(channel_id, fallback_content).await;
        };

        // まず返信形式で送信を試みる
        let reference = MessageReference::from((serenity_channel_id, serenity_message_id));
        let reply_message = content
            .clone()
            .into_serenity_message()
            .reference_message(reference);

        match serenity_channel_id
            .send_message(&self.http, reply_message)
            .await
        {
            Ok(sent_message) => {
                tracing::debug!(
                    channel_id = %channel_id,
                    reply_to = %reply_to_message_id,
                    "返信形式でメッセージを送信しました"
                );
                Ok(DiscordMessageId::new(sent_message.id.get()))
            }
            Err(e) => {
                // 返信失敗をログに記録
                tracing::warn!(
                    error = %e,
                    channel_id = %channel_id,
                    reply_to = %reply_to_message_id,
                    "返信形式でのメッセージ送信に失敗しました。通常メッセージとして再送信します"
                );

                // 文脈情報を付加して通常メッセージとして送信
                let fallback_content = if let Some(context) = fallback_context {
                    let original_text = content.text.clone().unwrap_or_default();
                    MessageContent::text(format!("【{context}】\n{original_text}"))
                } else {
                    content
                };

                let normal_message = fallback_content.into_serenity_message();
                let sent_message = serenity_channel_id
                    .send_message(&self.http, normal_message)
                    .await
                    .map_err(GatewayError::send_message_failed)?;

                Ok(DiscordMessageId::new(sent_message.id.get()))
            }
        }
    }
}

#[async_trait]
impl DiscordChannelGateway for PoiseDiscordGateway {
    async fn create_channel(
        &self,
        guild_id: DiscordGuildId,
        params: ChannelCreateParams,
    ) -> Result<DiscordChannelId, GatewayError> {
        let serenity_guild_id = GuildId::new(guild_id.get());

        let mut builder = CreateChannel::new(params.name).kind(params.kind.into());

        if let Some(parent_id) = params.parent_id {
            builder = builder.category(ChannelId::new(parent_id.get()));
        }

        if let Some(topic) = params.topic {
            builder = builder.topic(topic);
        }

        if let Some(nsfw) = params.nsfw {
            builder = builder.nsfw(nsfw);
        }

        if let Some(position) = params.position {
            builder = builder.position(position);
        }

        let channel = serenity_guild_id
            .create_channel(&self.http, builder)
            .await
            .map_err(GatewayError::create_channel_failed)?;

        Ok(DiscordChannelId::new(channel.id.get()))
    }

    async fn edit_channel(
        &self,
        channel_id: DiscordChannelId,
        params: ChannelEditParams,
    ) -> Result<(), GatewayError> {
        let serenity_channel_id = ChannelId::new(channel_id.get());

        let mut builder = EditChannel::new();

        if let Some(name) = params.name {
            builder = builder.name(name);
        }

        if let Some(topic) = params.topic {
            builder = builder.topic(topic);
        }

        if let Some(position) = params.position {
            builder = builder.position(position);
        }

        if let Some(nsfw) = params.nsfw {
            builder = builder.nsfw(nsfw);
        }

        serenity_channel_id
            .edit(&self.http, builder)
            .await
            .map_err(GatewayError::edit_channel_failed)?;

        Ok(())
    }

    async fn delete_channel(&self, channel_id: DiscordChannelId) -> Result<(), GatewayError> {
        let serenity_channel_id = ChannelId::new(channel_id.get());

        serenity_channel_id
            .delete(&self.http)
            .await
            .map_err(GatewayError::delete_channel_failed)?;

        Ok(())
    }

    async fn get_channel(&self, channel_id: DiscordChannelId) -> Result<ChannelData, GatewayError> {
        let serenity_channel_id = ChannelId::new(channel_id.get());

        let channel = serenity_channel_id
            .to_channel(&self.http)
            .await
            .map_err(GatewayError::get_channel_failed)?;

        match channel {
            poise::serenity_prelude::Channel::Guild(gc) => Ok(ChannelData {
                id: DiscordChannelId::new(gc.id.get()),
                guild_id: Some(DiscordGuildId::new(gc.guild_id.get())),
                name: gc.name,
                kind: ChannelKind::from(gc.kind),
                parent_id: gc.parent_id.map(|id| DiscordChannelId::new(id.get())),
                topic: gc.topic,
                position: Some(i32::from(gc.position)),
            }),
            poise::serenity_prelude::Channel::Private(pc) => Ok(ChannelData {
                id: DiscordChannelId::new(pc.id.get()),
                guild_id: None,
                name: pc.recipient.name.clone(),
                kind: ChannelKind::Text,
                parent_id: None,
                topic: None,
                position: None,
            }),
            _ => Err(GatewayError::GetChannelFailed(
                "Unknown channel type".to_string(),
            )),
        }
    }
}

#[async_trait]
impl DiscordInteractionGateway for PoiseDiscordGateway {
    async fn defer_interaction(
        &self,
        _interaction_id: DiscordInteractionId,
        _interaction_token: &str,
    ) -> Result<(), GatewayError> {
        // インタラクション遅延はComponentInteractionオブジェクトから直接呼び出す必要があるため、
        // このメソッドは現在の設計では使用しない。
        // 代わりに、Events層でComponentInteraction.defer()を呼び出す。
        Err(GatewayError::Internal(
            "defer_interaction should be called from ComponentInteraction directly".to_string(),
        ))
    }

    async fn respond_to_interaction(
        &self,
        _interaction_id: DiscordInteractionId,
        _interaction_token: &str,
        _response: InteractionResponse,
    ) -> Result<(), GatewayError> {
        // インタラクション応答はComponentInteractionオブジェクトから直接呼び出す必要があるため、
        // このメソッドは現在の設計では使用しない。
        Err(GatewayError::Internal(
            "respond_to_interaction should be called from ComponentInteraction directly"
                .to_string(),
        ))
    }

    async fn edit_interaction_response(
        &self,
        _interaction_id: DiscordInteractionId,
        _interaction_token: &str,
        _response: InteractionResponse,
    ) -> Result<(), GatewayError> {
        // インタラクション応答編集はComponentInteractionオブジェクトから直接呼び出す必要があるため、
        // このメソッドは現在の設計では使用しない。
        Err(GatewayError::Internal(
            "edit_interaction_response should be called from ComponentInteraction directly"
                .to_string(),
        ))
    }
}

#[async_trait]
impl DiscordReactionGateway for PoiseDiscordGateway {
    async fn get_reaction_users(
        &self,
        channel_id: DiscordChannelId,
        message_id: DiscordMessageId,
        emoji: ReactionEmoji,
        limit: Option<u8>,
    ) -> Result<Vec<DiscordUserId>, GatewayError> {
        let serenity_channel_id = ChannelId::new(channel_id.get());
        let serenity_message_id = serenity_message_id(message_id)?;
        let reaction_type: ReactionType = emoji.into();

        let users = serenity_channel_id
            .reaction_users(&self.http, serenity_message_id, reaction_type, limit, None)
            .await
            .map_err(GatewayError::get_reactions_failed)?;

        // ボットユーザーを除外する（bot自身がリアクションを追加するため参加者に含まれてしまう）
        Ok(users
            .into_iter()
            .filter(|u| !u.bot)
            .map(|u| DiscordUserId::new(u.id.get()))
            .collect())
    }

    async fn add_reaction(
        &self,
        channel_id: DiscordChannelId,
        message_id: DiscordMessageId,
        emoji: ReactionEmoji,
    ) -> Result<(), GatewayError> {
        let serenity_channel_id = ChannelId::new(channel_id.get());
        let serenity_message_id = serenity_message_id(message_id)?;
        let reaction_type: ReactionType = emoji.into();

        self.http
            .create_reaction(serenity_channel_id, serenity_message_id, &reaction_type)
            .await
            .map_err(GatewayError::add_reaction_failed)?;

        Ok(())
    }

    async fn remove_own_reaction(
        &self,
        channel_id: DiscordChannelId,
        message_id: DiscordMessageId,
        emoji: ReactionEmoji,
    ) -> Result<(), GatewayError> {
        let serenity_channel_id = ChannelId::new(channel_id.get());
        let serenity_message_id = serenity_message_id(message_id)?;
        let reaction_type: ReactionType = emoji.into();

        self.http
            .delete_reaction_me(serenity_channel_id, serenity_message_id, &reaction_type)
            .await
            .map_err(|e| GatewayError::Internal(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serenity_message_id_opt_zero_is_none() {
        assert!(serenity_message_id_opt(DiscordMessageId::new(0)).is_none());
    }

    #[test]
    fn test_serenity_message_id_zero_is_error() {
        let result = serenity_message_id(DiscordMessageId::new(0));
        assert!(matches!(result, Err(GatewayError::Internal(_))));
    }
}

#[async_trait]
impl DiscordGuildGateway for PoiseDiscordGateway {
    async fn get_member(
        &self,
        guild_id: DiscordGuildId,
        user_id: DiscordUserId,
    ) -> Result<GuildMember, GatewayError> {
        let serenity_guild_id = GuildId::new(guild_id.get());
        let serenity_user_id = UserId::new(user_id.get());

        let member = serenity_guild_id
            .member(&self.http, serenity_user_id)
            .await
            .map_err(GatewayError::get_member_failed)?;

        Ok(GuildMember {
            user_id: DiscordUserId::new(member.user.id.get()),
            guild_id: DiscordGuildId::new(guild_id.get()),
            nickname: member.nick,
            roles: member
                .roles
                .iter()
                .map(|r| DiscordRoleId::new(r.get()))
                .collect(),
            joined_at: member.joined_at.map(|t| t.to_string()),
            premium_since: member.premium_since.map(|t| t.to_string()),
            deaf: member.deaf,
            mute: member.mute,
        })
    }

    async fn get_roles(&self, guild_id: DiscordGuildId) -> Result<Vec<GuildRole>, GatewayError> {
        let serenity_guild_id = GuildId::new(guild_id.get());

        let roles = serenity_guild_id
            .roles(&self.http)
            .await
            .map_err(GatewayError::get_roles_failed)?;

        Ok(roles
            .into_values()
            .map(|role| GuildRole {
                id: DiscordRoleId::new(role.id.get()),
                name: role.name,
                color: role.colour.0,
                hoist: role.hoist,
                position: i32::from(role.position),
                permissions: role.permissions.bits(),
                managed: role.managed,
                mentionable: role.mentionable,
            })
            .collect())
    }

    async fn get_emojis(&self, guild_id: DiscordGuildId) -> Result<Vec<GuildEmoji>, GatewayError> {
        let serenity_guild_id = GuildId::new(guild_id.get());

        let emojis = serenity_guild_id
            .emojis(&self.http)
            .await
            .map_err(GatewayError::get_emojis_failed)?;

        Ok(emojis
            .into_iter()
            .map(|emoji| GuildEmoji {
                id: DiscordEmojiId::new(emoji.id.get()),
                name: emoji.name,
                animated: emoji.animated,
                roles: emoji
                    .roles
                    .iter()
                    .map(|r| DiscordRoleId::new(r.get()))
                    .collect(),
                available: emoji.available,
            })
            .collect())
    }
}
