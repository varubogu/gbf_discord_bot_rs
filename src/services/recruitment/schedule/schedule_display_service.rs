use poise::serenity_prelude::AutocompleteChoice;

use crate::models::entities::battle_recruitment_schedule_days;
use crate::models::entities::battle_recruitment_schedules;

/// スケジュール表示用のユーティリティ（サービス層）
pub struct ScheduleDisplayService;

impl ScheduleDisplayService {
    /// 曜日リストを表示用にフォーマット
    pub fn format_days_for_display(
        days: &[battle_recruitment_schedule_days::Model],
    ) -> String {
        if days.is_empty() {
            return "なし".to_string();
        }

        // 毎日かチェック（0: 毎日）
        if days.iter().any(|d| d.day_of_week == 0) {
            return "毎日".to_string();
        }

        // 曜日をソート
        let mut day_numbers: Vec<i32> = days.iter().map(|d| d.day_of_week).collect();
        day_numbers.sort();

        // 曜日を文字列に変換
        let day_strs: Vec<String> = day_numbers
            .iter()
            .map(|&day| match day {
                1 => "月",
                2 => "火",
                3 => "水",
                4 => "木",
                5 => "金",
                6 => "土",
                7 => "日",
                _ => "?",
            })
            .map(|s| s.to_string())
            .collect();

        day_strs.join("")
    }

    /// オートコンプリート用の候補へ変換（最大25件）
    pub fn to_autocomplete(
        schedules: &[(battle_recruitment_schedules::Model, Vec<battle_recruitment_schedule_days::Model>)],
    ) -> Vec<AutocompleteChoice> {
        let mut choices = Vec::new();

        for (schedule, days) in schedules.iter().take(25) {
            let days_str = Self::format_days_for_display(days);

            let time_str = format!(
                "{:02}:{:02}",
                schedule.quest_start_time.hour(),
                schedule.quest_start_time.minute()
            );

            let status = if schedule.is_enabled { "⭕" } else { "❌" };

            // 例: "⭕ 天元21時 (ID:1 火木土 22:00)"
            let display_name = format!(
                "{} {} (ID:{} {} {})",
                status, schedule.name, schedule.id, days_str, time_str
            );

            choices.push(AutocompleteChoice::new(display_name, schedule.id as i64));
        }

        choices
    }
}
