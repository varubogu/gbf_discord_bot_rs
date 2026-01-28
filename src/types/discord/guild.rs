//! ギルド関連のドメイン型
//!
//! Discordギルド（サーバー）に関するデータを定義する。

use super::{DiscordEmojiId, DiscordGuildId, DiscordRoleId, DiscordUserId};

/// ギルドメンバーデータ
#[derive(Debug, Clone)]
pub struct GuildMember {
    /// ユーザーID
    pub user_id: DiscordUserId,
    /// ギルドID
    pub guild_id: DiscordGuildId,
    /// ニックネーム
    pub nickname: Option<String>,
    /// ロールID一覧
    pub roles: Vec<DiscordRoleId>,
    /// 参加日時（ISO 8601形式）
    pub joined_at: Option<String>,
    /// サーバーブーストしているか
    pub premium_since: Option<String>,
    /// 聴覚障害者モードか
    pub deaf: bool,
    /// ミュートされているか
    pub mute: bool,
}

impl GuildMember {
    /// 指定されたロールを持っているか確認する
    pub fn has_role(&self, role_id: DiscordRoleId) -> bool {
        self.roles.contains(&role_id)
    }

    /// 表示名を取得する（ニックネーム優先）
    pub fn display_name(&self) -> Option<&str> {
        self.nickname.as_deref()
    }
}

/// ギルドロールデータ
#[derive(Debug, Clone)]
pub struct GuildRole {
    /// ロールID
    pub id: DiscordRoleId,
    /// ロール名
    pub name: String,
    /// カラーコード（RGB）
    pub color: u32,
    /// ホイスト（オンラインメンバー一覧で分離表示）されるか
    pub hoist: bool,
    /// ポジション
    pub position: i32,
    /// 権限ビットフラグ
    pub permissions: u64,
    /// 管理対象ロール（BOT等）か
    pub managed: bool,
    /// メンション可能か
    pub mentionable: bool,
}

impl GuildRole {
    /// 管理者権限を持っているか確認する
    pub fn is_administrator(&self) -> bool {
        // Administrator permission bit
        const ADMINISTRATOR: u64 = 0x0000000008;
        self.permissions & ADMINISTRATOR != 0
    }
}

/// ギルド絵文字データ
#[derive(Debug, Clone)]
pub struct GuildEmoji {
    /// 絵文字ID
    pub id: DiscordEmojiId,
    /// 絵文字名
    pub name: String,
    /// アニメーションかどうか
    pub animated: bool,
    /// 使用可能なロールID一覧（空の場合は全員使用可能）
    pub roles: Vec<DiscordRoleId>,
    /// 利用可能かどうか
    pub available: bool,
}

impl GuildEmoji {
    /// メンション形式の文字列を取得する
    pub fn mention(&self) -> String {
        if self.animated {
            format!("<a:{}:{}>", self.name, self.id.get())
        } else {
            format!("<:{}:{}>", self.name, self.id.get())
        }
    }

    /// API用の識別子を取得する
    pub fn to_api_string(&self) -> String {
        format!("{}:{}", self.name, self.id.get())
    }
}

/// ギルドデータ
#[derive(Debug, Clone)]
pub struct GuildData {
    /// ギルドID
    pub id: DiscordGuildId,
    /// ギルド名
    pub name: String,
    /// オーナーID
    pub owner_id: DiscordUserId,
    /// アイコンハッシュ
    pub icon: Option<String>,
    /// メンバー数
    pub member_count: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guild_member_has_role() {
        let member = GuildMember {
            user_id: DiscordUserId::new(123),
            guild_id: DiscordGuildId::new(456),
            nickname: Some("TestUser".to_string()),
            roles: vec![DiscordRoleId::new(111), DiscordRoleId::new(222)],
            joined_at: None,
            premium_since: None,
            deaf: false,
            mute: false,
        };

        assert!(member.has_role(DiscordRoleId::new(111)));
        assert!(!member.has_role(DiscordRoleId::new(333)));
    }

    #[test]
    fn test_guild_role_is_administrator() {
        let admin_role = GuildRole {
            id: DiscordRoleId::new(1),
            name: "Admin".to_string(),
            color: 0xFF0000,
            hoist: true,
            position: 10,
            permissions: 0x0000000008, // Administrator
            managed: false,
            mentionable: true,
        };

        let normal_role = GuildRole {
            id: DiscordRoleId::new(2),
            name: "Member".to_string(),
            color: 0x00FF00,
            hoist: false,
            position: 1,
            permissions: 0x0000000001, // Create Instant Invite only
            managed: false,
            mentionable: false,
        };

        assert!(admin_role.is_administrator());
        assert!(!normal_role.is_administrator());
    }

    #[test]
    fn test_guild_emoji_mention() {
        let emoji = GuildEmoji {
            id: DiscordEmojiId::new(123456),
            name: "myemoji".to_string(),
            animated: false,
            roles: vec![],
            available: true,
        };

        assert_eq!(emoji.mention(), "<:myemoji:123456>");

        let animated_emoji = GuildEmoji {
            id: DiscordEmojiId::new(789012),
            name: "animoji".to_string(),
            animated: true,
            roles: vec![],
            available: true,
        };

        assert_eq!(animated_emoji.mention(), "<a:animoji:789012>");
    }
}
