//! スケジュールプレゼンター
//!
//! 定期募集スケジュールの表示（一覧、詳細、オートコンプリート等）を担当する。
//! Service層からUIビルダー依存を除去するために使用する。

use crate::types::discord::{AutocompleteOption, EmbedContent};
use chrono::{TimeZone, Timelike};
use chrono_tz::Tz;

/// スケジュール表示情報
#[derive(Debug, Clone)]
pub struct ScheduleDisplayInfo {
    /// スケジュールID
    pub id: i32,
    /// スケジュール名
    pub name: String,
    /// クエスト名
    pub quest_name: String,
    /// 曜日文字列（"月火水" や "毎日" 等）
    pub days_display: String,
    /// 開始時刻（ローカルタイムゾーン）
    pub time_display: String,
    /// 有効/無効
    pub is_enabled: bool,
}

/// スケジュール作成結果
#[derive(Debug, Clone)]
pub struct ScheduleCreationDisplayInfo {
    /// スケジュールID
    pub schedule_id: i32,
    /// スケジュール名
    pub schedule_name: String,
    /// クエストID
    pub quest_id: i32,
    /// クエスト名
    pub quest_name: String,
    /// 攻略方法名
    pub battle_style_name: String,
    /// 曜日表示
    pub days_display: String,
    /// タイムゾーン名
    pub timezone: String,
    /// クエスト開始時刻表示
    pub quest_start_time: String,
    /// 募集開始日オフセット
    pub recruit_start_day_offset: i32,
    /// 募集開始時刻表示
    pub recruit_start_time: String,
    /// 解散時刻表示
    pub dismissal_times: Option<String>,
    /// 備考
    pub note: Option<String>,
}

/// スケジュール表示を担当するPresenter
///
/// 定期募集スケジュールの一覧、詳細表示、オートコンプリート等を生成する。
/// poise/serenity型は使用せず、ドメインモデルを返す。
pub struct SchedulePresenter;

impl SchedulePresenter {
    /// 曜日リストを表示用にフォーマットする
    ///
    /// # Arguments
    ///
    /// * `day_numbers` - 曜日番号一覧（0: 毎日, 1: 月, 2: 火, ..., 7: 日）
    ///
    /// # Returns
    ///
    /// 曜日表示文字列（"月火水" や "毎日" 等）
    pub fn format_days(day_numbers: &[i32]) -> String {
        if day_numbers.is_empty() {
            return "なし".to_string();
        }

        // 毎日かチェック（0: 毎日）
        if day_numbers.contains(&0) {
            return "毎日".to_string();
        }

        // 曜日をソート
        let mut sorted_days = day_numbers.to_vec();
        sorted_days.sort();

        // 曜日を文字列に変換
        let day_strs: Vec<&str> = sorted_days
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
            .collect();

        day_strs.join("")
    }

    /// スケジュール一覧Embedを生成する
    ///
    /// # Arguments
    ///
    /// * `schedules` - スケジュール表示情報一覧
    ///
    /// # Returns
    ///
    /// EmbedContent
    pub fn create_schedule_list_embed(schedules: &[ScheduleDisplayInfo]) -> EmbedContent {
        let mut embed = EmbedContent::new()
            .with_title("📅 定期募集スケジュール一覧")
            .with_color(0x3498db);

        if schedules.is_empty() {
            embed = embed.with_description("登録されているスケジュールはありません");
        } else {
            for schedule in schedules {
                let status = if schedule.is_enabled { "⭕" } else { "❌" };
                let field_name = format!("{} {} (ID:{})", status, schedule.name, schedule.id);
                let field_value = format!(
                    "クエスト: {}\n曜日: {} / 時刻: {}",
                    schedule.quest_name, schedule.days_display, schedule.time_display
                );
                embed = embed.with_field(field_name, field_value, false);
            }
        }

        embed
    }

    /// スケジュール選択用オートコンプリートオプションを生成する
    ///
    /// # Arguments
    ///
    /// * `schedules` - スケジュール表示情報一覧
    ///
    /// # Returns
    ///
    /// AutocompleteOptionのVec
    pub fn create_schedule_autocomplete(
        schedules: &[ScheduleDisplayInfo],
    ) -> Vec<AutocompleteOption> {
        schedules
            .iter()
            .take(25)
            .map(|schedule| {
                let status = if schedule.is_enabled { "⭕" } else { "❌" };
                let display_name = format!(
                    "{} {} (ID:{} {} {})",
                    status,
                    schedule.name,
                    schedule.id,
                    schedule.days_display,
                    schedule.time_display
                );
                AutocompleteOption::new(display_name, schedule.id.to_string())
            })
            .collect()
    }

    /// スケジュール作成成功Embedを生成する
    ///
    /// # Arguments
    ///
    /// * `info` - スケジュール作成結果情報
    /// * `user_id` - 作成者のユーザーID
    ///
    /// # Returns
    ///
    /// EmbedContent
    pub fn create_schedule_creation_embed(
        info: &ScheduleCreationDisplayInfo,
        user_id: u64,
    ) -> EmbedContent {
        let dismissal_display = info.dismissal_times.as_deref().unwrap_or("-");
        let note_display = info.note.as_deref().unwrap_or("-");

        let description = format!(
            "**スケジュール名**: {}\n\
             **スケジュールID**: {}\n\
             **クエスト**: {} (ID: {})\n\
             **マルチ攻略方法**: {}\n\
             **対象曜日**: {} ({}タイムゾーン)\n\
             **クエスト開始時刻**: {}\n\
             **募集開始**: {}日前の{}\n\
             **解散時刻**: {}\n\
             **備考**: {}\n\
             **作成者**: <@{}>\n\n\
             このスケジュールに基づいて、自動的に募集が投稿されます。\n\
             参加人数はクエストごとの設定を使用します。",
            info.schedule_name,
            info.schedule_id,
            info.quest_name,
            info.quest_id,
            info.battle_style_name,
            info.days_display,
            info.timezone,
            info.quest_start_time,
            info.recruit_start_day_offset,
            info.recruit_start_time,
            dismissal_display,
            note_display,
            user_id
        );

        EmbedContent::new()
            .with_title("✅ 定期募集スケジュールを作成しました")
            .with_description(description)
            .with_color(0x00ff00)
            .with_footer("スケジュールが正常に登録されました")
    }

    /// スケジュール削除成功Embedを生成する
    ///
    /// # Arguments
    ///
    /// * `schedule_name` - 削除されたスケジュール名
    ///
    /// # Returns
    ///
    /// EmbedContent
    pub fn create_schedule_deletion_embed(schedule_name: &str) -> EmbedContent {
        EmbedContent::new()
            .with_title("🗑️ スケジュールを削除しました")
            .with_description(format!("「{schedule_name}」を削除しました"))
            .with_color(0x00ff00)
    }

    /// スケジュール有効/無効切り替えEmbedを生成する
    ///
    /// # Arguments
    ///
    /// * `schedule_name` - スケジュール名
    /// * `is_enabled` - 新しい有効/無効状態
    ///
    /// # Returns
    ///
    /// EmbedContent
    pub fn create_schedule_toggle_embed(schedule_name: &str, is_enabled: bool) -> EmbedContent {
        let (status, color) = if is_enabled {
            ("有効", 0x00ff00)
        } else {
            ("無効", 0xff0000)
        };

        EmbedContent::new()
            .with_title(format!("🔄 スケジュールを{status}にしました"))
            .with_description(format!("「{schedule_name}」が{status}になりました"))
            .with_color(color)
    }

    /// UTC時刻をタイムゾーン適用してフォーマットする
    ///
    /// # Arguments
    ///
    /// * `hour` - UTCの時
    /// * `minute` - UTCの分
    /// * `timezone` - 適用するタイムゾーン
    ///
    /// # Returns
    ///
    /// "HH:MM" 形式の文字列
    pub fn format_time_with_timezone(hour: u32, minute: u32, timezone: &Tz) -> String {
        // NaiveTimeを仮の日付と組み合わせてDateTime<Utc>に変換
        let utc_datetime = match chrono::Utc
            .with_ymd_and_hms(2000, 1, 1, hour, minute, 0)
            .single()
        {
            Some(datetime) => datetime,
            None => {
                return format!("{:02}:{:02}", hour.min(23), minute.min(59));
            }
        };
        let local_datetime = utc_datetime.with_timezone(timezone);
        let hour = local_datetime.hour();
        let minute = local_datetime.minute();

        format!("{hour:02}:{minute:02}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_days_empty() {
        assert_eq!(SchedulePresenter::format_days(&[]), "なし");
    }

    #[test]
    fn test_format_days_everyday() {
        assert_eq!(SchedulePresenter::format_days(&[0]), "毎日");
        assert_eq!(SchedulePresenter::format_days(&[0, 1, 2]), "毎日");
    }

    #[test]
    fn test_format_days_specific() {
        assert_eq!(SchedulePresenter::format_days(&[1, 3, 5]), "月水金");
        assert_eq!(SchedulePresenter::format_days(&[6, 7]), "土日");
        // 順番がバラバラでもソートされる
        assert_eq!(SchedulePresenter::format_days(&[5, 3, 1]), "月水金");
    }

    #[test]
    fn test_create_schedule_list_embed_empty() {
        let embed = SchedulePresenter::create_schedule_list_embed(&[]);

        assert!(embed.title.as_ref().unwrap().contains("スケジュール一覧"));
        assert!(
            embed
                .description
                .as_ref()
                .unwrap()
                .contains("登録されているスケジュールはありません")
        );
    }

    #[test]
    fn test_create_schedule_list_embed_with_items() {
        let schedules = vec![
            ScheduleDisplayInfo {
                id: 1,
                name: "テストスケジュール".to_string(),
                quest_name: "テストクエスト".to_string(),
                days_display: "月水金".to_string(),
                time_display: "21:00".to_string(),
                is_enabled: true,
            },
            ScheduleDisplayInfo {
                id: 2,
                name: "無効スケジュール".to_string(),
                quest_name: "別クエスト".to_string(),
                days_display: "土日".to_string(),
                time_display: "22:00".to_string(),
                is_enabled: false,
            },
        ];

        let embed = SchedulePresenter::create_schedule_list_embed(&schedules);

        assert_eq!(embed.fields.len(), 2);
        assert!(embed.fields[0].name.contains("⭕"));
        assert!(embed.fields[1].name.contains("❌"));
    }

    #[test]
    fn test_create_schedule_autocomplete() {
        let schedules = vec![ScheduleDisplayInfo {
            id: 1,
            name: "テストスケジュール".to_string(),
            quest_name: "テストクエスト".to_string(),
            days_display: "月水金".to_string(),
            time_display: "21:00".to_string(),
            is_enabled: true,
        }];

        let options = SchedulePresenter::create_schedule_autocomplete(&schedules);

        assert_eq!(options.len(), 1);
        assert!(options[0].name.contains("⭕"));
        assert_eq!(options[0].value, "1");
    }

    #[test]
    fn test_create_schedule_creation_embed() {
        let info = ScheduleCreationDisplayInfo {
            schedule_id: 1,
            schedule_name: "テストスケジュール".to_string(),
            quest_id: 1,
            quest_name: "テストクエスト".to_string(),
            battle_style_name: "6属性".to_string(),
            days_display: "月水金".to_string(),
            timezone: "JST".to_string(),
            quest_start_time: "21:00".to_string(),
            recruit_start_day_offset: 1,
            recruit_start_time: "12:00".to_string(),
            dismissal_times: Some("20:30".to_string()),
            note: Some("テスト備考".to_string()),
        };

        let embed = SchedulePresenter::create_schedule_creation_embed(&info, 123456789);

        assert!(embed.title.as_ref().unwrap().contains("作成しました"));
        assert!(
            embed
                .description
                .as_ref()
                .unwrap()
                .contains("テストスケジュール")
        );
        assert!(embed.description.as_ref().unwrap().contains("<@123456789>"));
    }

    #[test]
    fn test_create_schedule_deletion_embed() {
        let embed = SchedulePresenter::create_schedule_deletion_embed("削除スケジュール");

        assert!(embed.title.as_ref().unwrap().contains("削除"));
        assert!(
            embed
                .description
                .as_ref()
                .unwrap()
                .contains("削除スケジュール")
        );
    }

    #[test]
    fn test_create_schedule_toggle_embed() {
        let embed_enabled = SchedulePresenter::create_schedule_toggle_embed("テスト", true);
        assert!(embed_enabled.title.as_ref().unwrap().contains("有効"));
        assert_eq!(embed_enabled.color, Some(0x00ff00));

        let embed_disabled = SchedulePresenter::create_schedule_toggle_embed("テスト", false);
        assert!(embed_disabled.title.as_ref().unwrap().contains("無効"));
        assert_eq!(embed_disabled.color, Some(0xff0000));
    }

    #[test]
    fn test_format_time_with_timezone() {
        // JST (UTC+9) でテスト
        let jst: Tz = "Asia/Tokyo".parse().unwrap();

        // UTC 12:00 → JST 21:00
        let result = SchedulePresenter::format_time_with_timezone(12, 0, &jst);
        assert_eq!(result, "21:00");

        // UTC 15:30 → JST 00:30 (翌日)
        let result = SchedulePresenter::format_time_with_timezone(15, 30, &jst);
        assert_eq!(result, "00:30");
    }
}
