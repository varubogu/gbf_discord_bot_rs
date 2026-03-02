//! リアクション関連のドメイン型
//!
//! Discordリアクションに関するデータを定義する。

use super::DiscordEmojiId;

/// リアクション絵文字
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReactionEmoji {
    /// Unicode絵文字
    Unicode(String),
    /// カスタム絵文字
    Custom {
        /// 絵文字ID
        id: DiscordEmojiId,
        /// 絵文字名
        name: String,
        /// アニメーションかどうか
        animated: bool,
    },
}

impl ReactionEmoji {
    /// Unicode絵文字を作成する
    pub fn unicode(emoji: impl Into<String>) -> Self {
        Self::Unicode(emoji.into())
    }

    /// カスタム絵文字を作成する
    pub fn custom(id: DiscordEmojiId, name: impl Into<String>) -> Self {
        Self::Custom {
            id,
            name: name.into(),
            animated: false,
        }
    }

    /// アニメーション付きカスタム絵文字を作成する
    pub fn custom_animated(id: DiscordEmojiId, name: impl Into<String>) -> Self {
        Self::Custom {
            id,
            name: name.into(),
            animated: true,
        }
    }

    /// Unicode絵文字かどうか
    pub fn is_unicode(&self) -> bool {
        matches!(self, Self::Unicode(_))
    }

    /// カスタム絵文字かどうか
    pub fn is_custom(&self) -> bool {
        matches!(self, Self::Custom { .. })
    }

    /// 絵文字の文字列表現を取得する
    ///
    /// Discord APIで使用する形式で返す。
    pub fn to_api_string(&self) -> String {
        match self {
            Self::Unicode(emoji) => emoji.clone(),
            Self::Custom { id, name, .. } => format!("{}:{}", name, id.get()),
        }
    }
}

impl std::fmt::Display for ReactionEmoji {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unicode(emoji) => write!(f, "{emoji}"),
            Self::Custom { name, id, animated } => {
                if *animated {
                    write!(f, "<a:{}:{}>", name, id.get())
                } else {
                    write!(f, "<:{}:{}>", name, id.get())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unicode_emoji() {
        let emoji = ReactionEmoji::unicode("👍");

        assert!(emoji.is_unicode());
        assert!(!emoji.is_custom());
        assert_eq!(emoji.to_api_string(), "👍");
        assert_eq!(format!("{emoji}"), "👍");
    }

    #[test]
    fn test_custom_emoji() {
        let emoji = ReactionEmoji::custom(DiscordEmojiId::new(123456), "myemoji");

        assert!(!emoji.is_unicode());
        assert!(emoji.is_custom());
        assert_eq!(emoji.to_api_string(), "myemoji:123456");
        assert_eq!(format!("{emoji}"), "<:myemoji:123456>");
    }

    #[test]
    fn test_animated_custom_emoji() {
        let emoji = ReactionEmoji::custom_animated(DiscordEmojiId::new(789012), "animated");

        assert_eq!(format!("{emoji}"), "<a:animated:789012>");
    }
}
