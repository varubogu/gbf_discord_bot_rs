use chrono::{TimeZone, Timelike};
use chrono_tz::Tz;
use poise::serenity_prelude::{AutocompleteChoice, CreateEmbed, CreateEmbedFooter};

use crate::models::entities::battle_recruitment_schedule_days;
use crate::models::entities::battle_recruitment_schedules;
use crate::services::recruitment::schedule::ScheduleCreationResult;

/// スケジュール表示用のユーティリティ（サービス層）
pub struct ScheduleDisplayService;

impl ScheduleDisplayService {
    /// 曜日リストを表示用にフォーマット
    pub fn format_days_for_display(days: &[battle_recruitment_schedule_days::Model]) -> String {
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
    /// タイムゾーンを適用して時刻を表示
    pub fn to_autocomplete(
        schedules: &[(
            battle_recruitment_schedules::Model,
            Vec<battle_recruitment_schedule_days::Model>,
        )],
        timezone: &Tz,
    ) -> Vec<AutocompleteChoice> {
        let mut choices = Vec::new();

        for (schedule, days) in schedules.iter().take(25) {
            let days_str = Self::format_days_for_display(days);

            // UTC時刻をタイムゾーンに変換
            // NaiveTimeを仮の日付（2000-01-01）と組み合わせてDateTime<Utc>に変換し、
            // その後タイムゾーンに変換して時刻部分を抽出
            let utc_datetime = chrono::Utc
                .with_ymd_and_hms(
                    2000,
                    1,
                    1,
                    schedule.quest_start_time.hour() as u32,
                    schedule.quest_start_time.minute() as u32,
                    0,
                )
                .unwrap();
            let local_datetime = utc_datetime.with_timezone(timezone);

            let time_str = format!(
                "{:02}:{:02}",
                local_datetime.hour(),
                local_datetime.minute()
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

    /// 定期募集スケジュール作成成功時の埋め込みを生成
    pub fn build_creation_embed(result: &ScheduleCreationResult, user_id: u64) -> CreateEmbed {
        let description = format!(
            "**スケジュール名**: {}\n\
             **スケジュールID**: {}\n\
             **クエスト**: {} (ID: {})\n\
             **マルチ攻略方法**: {}\n\
             **対象曜日**: {} ({}タイムゾーン)\n\
             **クエスト開始時刻**: {}\n\
             **募集開始**: {}日前の{}\n\
             **備考**: {}\n\
             **作成者**: <@{}>\n\n\
             このスケジュールに基づいて、自動的に募集が投稿されます。\n\
             参加人数はクエストごとの設定を使用します。",
            result.schedule_name,
            result.schedule_id,
            result.quest_name,
            result.quest_id,
            result.battle_style_name,
            result.days_display,
            result.timezone,
            result.quest_start_time,
            result.recruit_start_day_offset,
            result.recruit_start_time,
            result.note.as_ref().unwrap_or(&"-".to_string()),
            user_id
        );

        CreateEmbed::default()
            .title("✅ 定期募集スケジュールを作成しました")
            .description(description)
            .color(0x00ff00)
            .footer(CreateEmbedFooter::new("スケジュールが正常に登録されました"))
    }
}
