//! クエストメッセージビルダー
//!
//! 1クエスト1メッセージ形式のUIを構築する

use crate::types::BattleStyleId;
use poise::serenity_prelude::{
    ButtonStyle, CreateActionRow, CreateButton, CreateMessage, CreateSelectMenu,
    CreateSelectMenuKind, CreateSelectMenuOption, EditMessage,
};

/// 6属性の数
const SIX_ELEMENT_COUNT: usize = 6;

/// 属性情報
#[derive(Debug, Clone)]
pub struct BattleStyleInfo {
    pub id: i32,
    pub name: String,
    pub emoji: &'static str,
}

/// 6属性の定義（ID, 名前, 絵文字）
pub fn get_six_elements() -> Vec<BattleStyleInfo> {
    vec![
        BattleStyleInfo {
            id: 1,
            name: "火属性".to_string(),
            emoji: "🔥",
        },
        BattleStyleInfo {
            id: 2,
            name: "水属性".to_string(),
            emoji: "💧",
        },
        BattleStyleInfo {
            id: 3,
            name: "土属性".to_string(),
            emoji: "🌍",
        },
        BattleStyleInfo {
            id: 4,
            name: "風属性".to_string(),
            emoji: "💨",
        },
        BattleStyleInfo {
            id: 5,
            name: "光属性".to_string(),
            emoji: "✨",
        },
        BattleStyleInfo {
            id: 6,
            name: "闇属性".to_string(),
            emoji: "🌑",
        },
    ]
}

/// クエストメッセージビルダー
///
/// メッセージは全ユーザーで共有されるため、ボタン/セレクトメニューの見た目は
/// ユーザーの状態に依存せず、常に同じ表示になる。
/// ユーザーごとの状態はエフェメラル応答で通知する。
pub struct QuestMessageBuilder {
    guild_id: u64,
    quest_id: i32,
    quest_name: String,
    /// クエストのデフォルト戦術ID（6属性クエスト判定に使用）
    default_battle_style_id: i32,
}

impl QuestMessageBuilder {
    /// 新しいビルダーを作成
    pub fn new(guild_id: u64, quest_id: i32, quest_name: String) -> Self {
        Self {
            guild_id,
            quest_id,
            quest_name,
            default_battle_style_id: 0,
        }
    }

    /// デフォルト戦術IDを設定（6属性クエスト判定に使用）
    pub fn with_default_battle_style_id(mut self, id: i32) -> Self {
        self.default_battle_style_id = id;
        self
    }

    /// 6属性クエストかどうかを判定
    pub fn is_six_element_quest(&self) -> bool {
        BattleStyleId::is_six_elements(self.default_battle_style_id)
    }

    /// メッセージを構築（新規作成用）
    pub fn build(self) -> CreateMessage {
        let action_row = if self.is_six_element_quest() {
            self.build_element_select_menu()
        } else {
            self.build_participation_button()
        };

        let content = format!("🎮 **{}**", self.quest_name);

        CreateMessage::new()
            .content(content)
            .components(vec![action_row])
    }

    /// EditMessageを構築（既存メッセージの更新用）
    pub fn build_edit(self) -> EditMessage {
        let action_row = if self.is_six_element_quest() {
            self.build_element_select_menu()
        } else {
            self.build_participation_button()
        };

        let content = format!("🎮 **{}**", self.quest_name);

        EditMessage::new()
            .content(content)
            .components(vec![action_row])
    }

    /// 参加ボタンを構築（属性指定なしクエスト用）
    ///
    /// メッセージは全ユーザーで共有されるため、ボタンの見た目は常に同じ。
    /// ユーザーごとの参加状態はエフェメラル応答で通知する。
    fn build_participation_button(&self) -> CreateActionRow {
        let custom_id = format!("auto_quest_join:{}:{}", self.guild_id, self.quest_id);

        let button = CreateButton::new(custom_id)
            .label("参加する")
            .style(ButtonStyle::Primary);

        CreateActionRow::Buttons(vec![button])
    }

    /// 属性選択セレクトメニューを構築（6属性クエスト用）
    ///
    /// メッセージは全ユーザーで共有されるため、選択状態は表示しない。
    /// ユーザーごとの選択状態はエフェメラル応答で通知する。
    fn build_element_select_menu(&self) -> CreateActionRow {
        let custom_id = format!("auto_quest_element:{}:{}", self.guild_id, self.quest_id);

        // 6属性クエストでは全6属性を表示（選択状態なし）
        let elements = get_six_elements();
        let options: Vec<CreateSelectMenuOption> = elements
            .iter()
            .map(|element| {
                let label = format!("{} {}", element.emoji, element.name);
                CreateSelectMenuOption::new(label, element.id.to_string())
            })
            .collect();

        let select_menu =
            CreateSelectMenu::new(custom_id, CreateSelectMenuKind::String { options })
                .placeholder("属性を選択してください（複数選択可）")
                .min_values(0)
                .max_values(SIX_ELEMENT_COUNT as u8);

        CreateActionRow::SelectMenu(select_menu)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_six_element_quest() {
        // default_battle_style_id = 1 は6属性クエスト
        let builder =
            QuestMessageBuilder::new(1, 1, "テスト".to_string()).with_default_battle_style_id(1);

        assert!(builder.is_six_element_quest());
    }

    #[test]
    fn test_is_not_six_element_quest() {
        // default_battle_style_id = 0 は属性指定なしクエスト
        let builder =
            QuestMessageBuilder::new(1, 1, "テスト".to_string()).with_default_battle_style_id(0);

        assert!(!builder.is_six_element_quest());
    }

    #[test]
    fn test_build_with_button() {
        // 属性指定なしクエストはボタンを表示
        let builder =
            QuestMessageBuilder::new(1, 1, "テスト".to_string()).with_default_battle_style_id(0);

        // ビルドが成功することを確認
        let _message = builder.build();
    }

    #[test]
    fn test_build_with_select_menu() {
        // 6属性クエストはセレクトメニューを表示
        let builder =
            QuestMessageBuilder::new(1, 1, "テスト".to_string()).with_default_battle_style_id(1);

        // ビルドが成功することを確認
        let _message = builder.build();
    }
}
