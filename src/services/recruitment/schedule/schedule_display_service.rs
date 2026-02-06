//! スケジュール表示サービス
//!
//! スケジュール表示に必要なデータ変換を担当する。
//! UI構築はPresenter層に委譲する。

use chrono_tz::Tz;

use crate::models::entities::guild_master::battle_recruitment_schedule_days;
use crate::models::entities::guild_master::battle_recruitment_schedules;
use crate::presenter::{ScheduleCreationDisplayInfo, ScheduleDisplayInfo, SchedulePresenter};
use crate::services::recruitment::schedule::ScheduleCreationResult;
use crate::types::discord::{AutocompleteOption, EmbedContent};

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

        // 曜日番号を抽出してSchedulePresenterに委譲
        let day_numbers: Vec<i32> = days.iter().map(|d| d.day_of_week).collect();
        SchedulePresenter::format_days(&day_numbers)
    }

    /// エンティティからScheduleDisplayInfoに変換する
    fn to_display_info(
        schedule: &battle_recruitment_schedules::Model,
        days: &[battle_recruitment_schedule_days::Model],
        timezone: &Tz,
    ) -> ScheduleDisplayInfo {
        let days_display = Self::format_days_for_display(days);
        let time_display = SchedulePresenter::format_time_with_timezone(
            schedule.quest_start_time.hour() as u32,
            schedule.quest_start_time.minute() as u32,
            timezone,
        );

        ScheduleDisplayInfo {
            id: schedule.id,
            name: schedule.name.clone(),
            quest_name: String::new(), // 後で設定される場合がある
            days_display,
            time_display,
            is_enabled: schedule.is_enabled,
        }
    }

    /// オートコンプリート用の候補へ変換（最大25件）
    /// タイムゾーンを適用して時刻を表示
    pub fn to_autocomplete(
        schedules: &[(
            battle_recruitment_schedules::Model,
            Vec<battle_recruitment_schedule_days::Model>,
        )],
        timezone: &Tz,
    ) -> Vec<AutocompleteOption> {
        let display_infos: Vec<ScheduleDisplayInfo> = schedules
            .iter()
            .take(25)
            .map(|(schedule, days)| Self::to_display_info(schedule, days, timezone))
            .collect();

        SchedulePresenter::create_schedule_autocomplete(&display_infos)
    }

    /// 定期募集スケジュール作成成功時の埋め込みを生成
    pub fn build_creation_embed(result: &ScheduleCreationResult, user_id: u64) -> EmbedContent {
        // ScheduleCreationResultをScheduleCreationDisplayInfoに変換
        let display_info = ScheduleCreationDisplayInfo {
            schedule_id: result.schedule_id as i32,
            schedule_name: result.schedule_name.clone(),
            quest_id: result.quest_id,
            quest_name: result.quest_name.clone(),
            battle_style_name: result.battle_style_name.clone(),
            days_display: result.days_display.clone(),
            timezone: result.timezone.to_string(),
            quest_start_time: result.quest_start_time.clone(),
            recruit_start_day_offset: result.recruit_start_day_offset,
            recruit_start_time: result.recruit_start_time.clone(),
            dismissal_times: result.dismissal_times.clone(),
            note: result.note.clone(),
        };

        SchedulePresenter::create_schedule_creation_embed(&display_info, user_id)
    }
}
