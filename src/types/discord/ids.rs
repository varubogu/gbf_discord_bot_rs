//! Discord識別子のドメイン型
//!
//! poise/serenityのChannelId, MessageId等の代替となる型を定義する。
//! ビジネスロジック層ではこれらの型を使用し、Gateway実装層でのみserenity型に変換する。

use std::fmt;

/// DiscordチャンネルID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DiscordChannelId(pub u64);

impl DiscordChannelId {
    /// 新しいDiscordChannelIdを作成する
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// 内部の値を取得する
    pub fn get(&self) -> u64 {
        self.0
    }
}

impl From<u64> for DiscordChannelId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl From<i64> for DiscordChannelId {
    fn from(id: i64) -> Self {
        Self(id as u64)
    }
}

impl From<DiscordChannelId> for u64 {
    fn from(id: DiscordChannelId) -> Self {
        id.0
    }
}

impl From<DiscordChannelId> for i64 {
    fn from(id: DiscordChannelId) -> Self {
        id.0 as i64
    }
}

impl fmt::Display for DiscordChannelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// DiscordメッセージID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DiscordMessageId(pub u64);

impl DiscordMessageId {
    /// 新しいDiscordMessageIdを作成する
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// 内部の値を取得する
    pub fn get(&self) -> u64 {
        self.0
    }
}

impl From<u64> for DiscordMessageId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl From<i64> for DiscordMessageId {
    fn from(id: i64) -> Self {
        Self(id as u64)
    }
}

impl From<DiscordMessageId> for u64 {
    fn from(id: DiscordMessageId) -> Self {
        id.0
    }
}

impl From<DiscordMessageId> for i64 {
    fn from(id: DiscordMessageId) -> Self {
        id.0 as i64
    }
}

impl fmt::Display for DiscordMessageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// DiscordギルドID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DiscordGuildId(pub u64);

impl DiscordGuildId {
    /// 新しいDiscordGuildIdを作成する
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// 内部の値を取得する
    pub fn get(&self) -> u64 {
        self.0
    }
}

impl From<u64> for DiscordGuildId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl From<i64> for DiscordGuildId {
    fn from(id: i64) -> Self {
        Self(id as u64)
    }
}

impl From<DiscordGuildId> for u64 {
    fn from(id: DiscordGuildId) -> Self {
        id.0
    }
}

impl From<DiscordGuildId> for i64 {
    fn from(id: DiscordGuildId) -> Self {
        id.0 as i64
    }
}

impl fmt::Display for DiscordGuildId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// DiscordユーザーID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DiscordUserId(pub u64);

impl DiscordUserId {
    /// 新しいDiscordUserIdを作成する
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// 内部の値を取得する
    pub fn get(&self) -> u64 {
        self.0
    }
}

impl From<u64> for DiscordUserId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl From<i64> for DiscordUserId {
    fn from(id: i64) -> Self {
        Self(id as u64)
    }
}

impl From<DiscordUserId> for u64 {
    fn from(id: DiscordUserId) -> Self {
        id.0
    }
}

impl From<DiscordUserId> for i64 {
    fn from(id: DiscordUserId) -> Self {
        id.0 as i64
    }
}

impl fmt::Display for DiscordUserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// DiscordロールID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DiscordRoleId(pub u64);

impl DiscordRoleId {
    /// 新しいDiscordRoleIdを作成する
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// 内部の値を取得する
    pub fn get(&self) -> u64 {
        self.0
    }
}

impl From<u64> for DiscordRoleId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl From<i64> for DiscordRoleId {
    fn from(id: i64) -> Self {
        Self(id as u64)
    }
}

impl From<DiscordRoleId> for u64 {
    fn from(id: DiscordRoleId) -> Self {
        id.0
    }
}

impl From<DiscordRoleId> for i64 {
    fn from(id: DiscordRoleId) -> Self {
        id.0 as i64
    }
}

impl fmt::Display for DiscordRoleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// DiscordインタラクションID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DiscordInteractionId(pub u64);

impl DiscordInteractionId {
    /// 新しいDiscordInteractionIdを作成する
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// 内部の値を取得する
    pub fn get(&self) -> u64 {
        self.0
    }
}

impl From<u64> for DiscordInteractionId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl From<DiscordInteractionId> for u64 {
    fn from(id: DiscordInteractionId) -> Self {
        id.0
    }
}

impl fmt::Display for DiscordInteractionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Discord絵文字ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DiscordEmojiId(pub u64);

impl DiscordEmojiId {
    /// 新しいDiscordEmojiIdを作成する
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// 内部の値を取得する
    pub fn get(&self) -> u64 {
        self.0
    }
}

impl From<u64> for DiscordEmojiId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl From<DiscordEmojiId> for u64 {
    fn from(id: DiscordEmojiId) -> Self {
        id.0
    }
}

impl fmt::Display for DiscordEmojiId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discord_channel_id_conversion() {
        let id = DiscordChannelId::new(12345);
        assert_eq!(id.get(), 12345);

        let from_u64: DiscordChannelId = 12345u64.into();
        assert_eq!(from_u64, id);

        let to_u64: u64 = id.into();
        assert_eq!(to_u64, 12345);

        let from_i64: DiscordChannelId = 12345i64.into();
        assert_eq!(from_i64, id);
    }

    #[test]
    fn test_discord_message_id_conversion() {
        let id = DiscordMessageId::new(67890);
        assert_eq!(id.get(), 67890);
    }

    #[test]
    fn test_discord_guild_id_conversion() {
        let id = DiscordGuildId::new(11111);
        assert_eq!(id.get(), 11111);
    }

    #[test]
    fn test_discord_user_id_conversion() {
        let id = DiscordUserId::new(22222);
        assert_eq!(id.get(), 22222);
    }

    #[test]
    fn test_display_trait() {
        let channel_id = DiscordChannelId::new(12345);
        assert_eq!(format!("{channel_id}"), "12345");
    }
}
