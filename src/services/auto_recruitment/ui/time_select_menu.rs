//! 時間選択セレクトメニュー
//!
//! 日時チャンネルに表示する時間選択セレクトメニューを構築する

use poise::serenity_prelude::{
    CreateActionRow, CreateMessage, CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption,
};

/// 時間選択セレクトメニュービルダー
pub struct TimeSelectMenuBuilder {
    /// ギルドID
    guild_id: u64,
    /// 月
    month: i32,
    /// 日
    day: i32,
    /// 選択可能な開始時間
    start_hour: i32,
    /// 選択可能な終了時間
    end_hour: i32,
}

impl TimeSelectMenuBuilder {
    /// 新しいビルダーを作成
    pub fn new(guild_id: u64, month: i32, day: i32) -> Self {
        Self {
            guild_id,
            month,
            day,
            start_hour: 0,
            end_hour: 23,
        }
    }

    /// 選択可能な時間範囲を設定
    pub fn hour_range(mut self, start: i32, end: i32) -> Self {
        self.start_hour = start.clamp(0, 23);
        self.end_hour = end.clamp(0, 23);
        self
    }

    /// セレクトメニューを構築
    pub fn build(self) -> CreateActionRow {
        let options: Vec<CreateSelectMenuOption> = (self.start_hour..=self.end_hour)
            .map(|hour| CreateSelectMenuOption::new(format!("{}:00", hour), hour.to_string()))
            .collect();

        let max_values = options.len() as u8;
        let custom_id = format!(
            "auto_time_select:{}:{}:{}",
            self.guild_id, self.month, self.day
        );

        let select_menu =
            CreateSelectMenu::new(custom_id, CreateSelectMenuKind::String { options })
                .placeholder("参加可能な時間を選択してください（複数選択可）")
                .min_values(1)
                .max_values(max_values);

        CreateActionRow::SelectMenu(select_menu)
    }

    /// メッセージを構築
    pub fn build_message(self) -> CreateMessage {
        let month = self.month;
        let day = self.day;
        let action_row = self.build();

        CreateMessage::new()
            .content(format!(
                "**{}月{}日 参加可能時間選択**\n参加可能な時間を選択してください。複数選択可能です。",
                month, day
            ))
            .components(vec![action_row])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_select_menu_builder() {
        let builder = TimeSelectMenuBuilder::new(12345, 1, 15);
        let action_row = builder.build();

        // ActionRowが返されることを確認
        match action_row {
            CreateActionRow::SelectMenu(_) => {}
            _ => panic!("セレクトメニューが返されませんでした"),
        }
    }

    #[test]
    fn test_time_select_menu_builder_with_range() {
        let builder = TimeSelectMenuBuilder::new(12345, 1, 15).hour_range(9, 21);
        let _action_row = builder.build();

        // 範囲内の時間のみが選択肢に含まれることを確認
        // (実際のSelectMenuの中身を検証するのは難しいため、ビルドが成功することのみ確認)
    }
}
