//! チャンネル関連のドメイン型
//!
//! Discordチャンネルの作成・編集に必要なデータを表現する型を定義する。

use super::{DiscordChannelId, DiscordGuildId};

/// チャンネル作成パラメータ
#[derive(Debug, Clone)]
pub struct ChannelCreateParams {
    /// チャンネル名
    pub name: String,
    /// チャンネル種別
    pub kind: ChannelKind,
    /// 親カテゴリID
    pub parent_id: Option<DiscordChannelId>,
    /// トピック
    pub topic: Option<String>,
    /// NSFWフラグ
    pub nsfw: Option<bool>,
    /// ポジション
    pub position: Option<u16>,
}

impl ChannelCreateParams {
    /// テキストチャンネル作成パラメータを作成する
    pub fn text(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: ChannelKind::Text,
            parent_id: None,
            topic: None,
            nsfw: None,
            position: None,
        }
    }

    /// ボイスチャンネル作成パラメータを作成する
    pub fn voice(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: ChannelKind::Voice,
            parent_id: None,
            topic: None,
            nsfw: None,
            position: None,
        }
    }

    /// カテゴリ作成パラメータを作成する
    pub fn category(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: ChannelKind::Category,
            parent_id: None,
            topic: None,
            nsfw: None,
            position: None,
        }
    }

    /// 親カテゴリを設定する
    pub fn with_parent(mut self, parent_id: DiscordChannelId) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    /// トピックを設定する
    pub fn with_topic(mut self, topic: impl Into<String>) -> Self {
        self.topic = Some(topic.into());
        self
    }

    /// ポジションを設定する
    pub fn with_position(mut self, position: u16) -> Self {
        self.position = Some(position);
        self
    }
}

/// チャンネル編集パラメータ
#[derive(Debug, Clone, Default)]
pub struct ChannelEditParams {
    /// チャンネル名
    pub name: Option<String>,
    /// トピック
    pub topic: Option<String>,
    /// ポジション
    pub position: Option<u16>,
    /// 親カテゴリID
    pub parent_id: Option<Option<DiscordChannelId>>,
    /// NSFWフラグ
    pub nsfw: Option<bool>,
}

impl ChannelEditParams {
    /// 新しいChannelEditParamsを作成する
    pub fn new() -> Self {
        Self::default()
    }

    /// チャンネル名を設定する
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// トピックを設定する
    pub fn with_topic(mut self, topic: impl Into<String>) -> Self {
        self.topic = Some(topic.into());
        self
    }

    /// ポジションを設定する
    pub fn with_position(mut self, position: u16) -> Self {
        self.position = Some(position);
        self
    }

    /// 親カテゴリを設定する
    pub fn with_parent(mut self, parent_id: DiscordChannelId) -> Self {
        self.parent_id = Some(Some(parent_id));
        self
    }

    /// 親カテゴリを削除する
    pub fn without_parent(mut self) -> Self {
        self.parent_id = Some(None);
        self
    }
}

/// 取得したチャンネルデータ
#[derive(Debug, Clone)]
pub struct ChannelData {
    /// チャンネルID
    pub id: DiscordChannelId,
    /// ギルドID（DMチャンネルの場合はNone）
    pub guild_id: Option<DiscordGuildId>,
    /// チャンネル名
    pub name: String,
    /// チャンネル種別
    pub kind: ChannelKind,
    /// 親カテゴリID
    pub parent_id: Option<DiscordChannelId>,
    /// トピック
    pub topic: Option<String>,
    /// ポジション
    pub position: Option<i32>,
}

/// チャンネル種別
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelKind {
    /// テキストチャンネル
    Text,
    /// ボイスチャンネル
    Voice,
    /// カテゴリ
    Category,
    /// アナウンスメントチャンネル
    Announcement,
    /// スレッド（公開）
    PublicThread,
    /// スレッド（非公開）
    PrivateThread,
    /// ステージチャンネル
    Stage,
    /// フォーラムチャンネル
    Forum,
    /// 不明
    Unknown,
}

impl ChannelKind {
    /// テキストベースのチャンネルかどうか
    pub fn is_text_based(&self) -> bool {
        matches!(
            self,
            ChannelKind::Text
                | ChannelKind::Announcement
                | ChannelKind::PublicThread
                | ChannelKind::PrivateThread
        )
    }

    /// ボイスベースのチャンネルかどうか
    pub fn is_voice_based(&self) -> bool {
        matches!(self, ChannelKind::Voice | ChannelKind::Stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_create_params_builder() {
        let params = ChannelCreateParams::text("general")
            .with_topic("General discussion")
            .with_position(0);

        assert_eq!(params.name, "general");
        assert_eq!(params.kind, ChannelKind::Text);
        assert_eq!(params.topic, Some("General discussion".to_string()));
        assert_eq!(params.position, Some(0));
    }

    #[test]
    fn test_channel_edit_params_builder() {
        let params = ChannelEditParams::new()
            .with_name("new-name")
            .with_topic("New topic");

        assert_eq!(params.name, Some("new-name".to_string()));
        assert_eq!(params.topic, Some("New topic".to_string()));
    }

    #[test]
    fn test_channel_kind_is_text_based() {
        assert!(ChannelKind::Text.is_text_based());
        assert!(ChannelKind::Announcement.is_text_based());
        assert!(!ChannelKind::Voice.is_text_based());
        assert!(!ChannelKind::Category.is_text_based());
    }
}
