//! クエスト選択セレクトメニュー
//!
//! クエスト選択チャンネルに表示するセレクトメニューを構築する

use poise::serenity_prelude::{
    CreateActionRow, CreateMessage, CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption,
};

/// クエスト選択セレクトメニュービルダー
pub struct QuestSelectMenuBuilder {
    /// 選択肢となるクエストリスト (id, name)
    quests: Vec<(i32, String)>,
    /// ギルドID
    guild_id: u64,
    /// 最大選択数
    max_values: u8,
}

impl QuestSelectMenuBuilder {
    /// 新しいビルダーを作成
    pub fn new(guild_id: u64) -> Self {
        Self {
            quests: Vec::new(),
            guild_id,
            max_values: 25, // デフォルトは最大数
        }
    }

    /// クエストリストを設定
    pub fn quests(mut self, quests: Vec<(i32, String)>) -> Self {
        self.quests = quests;
        self
    }

    /// 最大選択数を設定
    pub fn max_values(mut self, max_values: u8) -> Self {
        self.max_values = max_values;
        self
    }

    /// セレクトメニューを構築
    ///
    /// 25件以上の場合は複数のセレクトメニューに分割
    pub fn build(self) -> Vec<CreateActionRow> {
        let mut action_rows = Vec::new();
        let chunk_size = 25;

        // 25件ごとにチャンク分割
        for (i, chunk) in self.quests.chunks(chunk_size).enumerate() {
            let options: Vec<CreateSelectMenuOption> = chunk
                .iter()
                .map(|(id, name)| CreateSelectMenuOption::new(name, id.to_string()))
                .collect();

            let max_values = std::cmp::min(self.max_values, options.len() as u8);

            // チャンクごとにカスタムIDを変える（複数メニューの場合）
            let custom_id = if i == 0 {
                format!("auto_quest_select:{}", self.guild_id)
            } else {
                format!("auto_quest_select:{}:{}", self.guild_id, i)
            };

            let select_menu =
                CreateSelectMenu::new(custom_id, CreateSelectMenuKind::String { options })
                    .placeholder("参加したいクエストを選択してください（複数選択可）")
                    .min_values(1)
                    .max_values(max_values);

            action_rows.push(CreateActionRow::SelectMenu(select_menu));
        }

        action_rows
    }

    /// メッセージを構築
    pub fn build_message(self) -> CreateMessage {
        let action_rows = self.build();

        CreateMessage::new()
            .content("**クエスト選択**\n参加したいクエストを選択してください。複数選択可能です。")
            .components(action_rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quest_select_menu_builder() {
        let quests = vec![
            (1, "テスト1".to_string()),
            (2, "テスト2".to_string()),
            (3, "テスト3".to_string()),
        ];

        let builder = QuestSelectMenuBuilder::new(12345)
            .quests(quests)
            .max_values(3);

        let action_rows = builder.build();
        assert_eq!(action_rows.len(), 1);
    }

    #[test]
    fn test_quest_select_menu_builder_over_25() {
        // 30件のクエストを作成
        let quests: Vec<(i32, String)> = (1..=30).map(|i| (i, format!("テスト{}", i))).collect();

        let builder = QuestSelectMenuBuilder::new(12345).quests(quests);

        let action_rows = builder.build();
        // 30件なので2つのセレクトメニューに分割
        assert_eq!(action_rows.len(), 2);
    }
}
