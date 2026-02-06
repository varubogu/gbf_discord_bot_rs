# Step 3: UIビルダーのプレゼンテーション層への分離

## 目的

Service層に存在するUIビルダーロジック（`CreateEmbed`, `CreateButton`等の構築）をプレゼンテーション層に移動し、Service層をUI非依存にする。

## 概要

```
┌─────────────────────────────────────────────────────────────┐
│                    Events Layer                              │
│              (poise commands, handlers)                      │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                 Presenter Layer (NEW)                        │
│     EmbedData, ComponentRow構築 → serenity型への変換         │
│         (ドメインモデル → Discord UIへの変換担当)             │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Facade Layer                              │
│           (ビジネスロジックオーケストレーション)               │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                   Service Layer                              │
│      (純粋なビジネスロジック - UI知識なし)                    │
└─────────────────────────────────────────────────────────────┘
```

## Presenter層の責務

1. **ドメインモデルからUIモデルへの変換**
   - ビジネスデータ → `EmbedData`, `ComponentRow`
2. **表示フォーマットの決定**
   - 日付フォーマット、色、レイアウト
3. **i18n/l10n対応**
   - ロケールに応じたテキスト生成
4. **UIコンポーネントの組み立て**
   - ボタン配置、セレクトメニュー構成

## 作成するPresenter

### 1. RecruitmentPresenter

募集関連の表示を担当。

```rust
// src/presenter/recruitment_presenter.rs

use crate::domain::models::{EmbedData, ComponentRow, ButtonData, ButtonStyle};
use crate::entity::battle_recruitments;

/// 募集情報の表示を担当するPresenter
pub struct RecruitmentPresenter;

impl RecruitmentPresenter {
    /// 募集Embedを生成
    pub fn create_recruitment_embed(
        recruitment: &battle_recruitments::Model,
        participants: &[ParticipantInfo],
        locale: &str,
    ) -> EmbedData {
        let title = Self::format_title(recruitment, locale);
        let description = Self::format_description(recruitment, locale);
        let color = Self::get_status_color(recruitment);

        EmbedData::new()
            .title(title)
            .description(description)
            .color(color)
            .field("参加者", Self::format_participants(participants), false)
            .field("開始時刻", Self::format_time(recruitment.start_time), true)
            .footer(Self::format_footer(recruitment))
    }

    /// 募集用ボタン行を生成
    pub fn create_recruitment_buttons(
        recruitment_id: i64,
        is_owner: bool,
        locale: &str,
    ) -> ComponentRow {
        let mut buttons = vec![
            ButtonData::primary(
                format!("join_{}", recruitment_id),
                Self::get_text("button.join", locale),
            ),
            ButtonData::secondary(
                format!("leave_{}", recruitment_id),
                Self::get_text("button.leave", locale),
            ),
        ];

        if is_owner {
            buttons.push(
                ButtonData::danger(
                    format!("cancel_{}", recruitment_id),
                    Self::get_text("button.cancel", locale),
                )
            );
        }

        ComponentRow::buttons(buttons)
    }

    /// キャンセル確認ダイアログを生成
    pub fn create_cancel_confirmation(
        recruitment_id: i64,
        locale: &str,
    ) -> (EmbedData, ComponentRow) {
        let embed = EmbedData::new()
            .title(Self::get_text("cancel.title", locale))
            .description(Self::get_text("cancel.confirm_message", locale))
            .color(0xff0000);

        let buttons = ComponentRow::buttons(vec![
            ButtonData::danger(
                format!("confirm_cancel_{}", recruitment_id),
                Self::get_text("button.confirm", locale),
            ),
            ButtonData::secondary(
                format!("abort_cancel_{}", recruitment_id),
                Self::get_text("button.abort", locale),
            ),
        ]);

        (embed, buttons)
    }

    // --- Private helpers ---

    fn format_title(recruitment: &battle_recruitments::Model, locale: &str) -> String {
        format!("{} - {}", recruitment.quest_name, recruitment.battle_style)
    }

    fn format_description(recruitment: &battle_recruitments::Model, locale: &str) -> String {
        // ビジネスロジックではなく表示フォーマットのみ
        format!("募集人数: {}/{}", recruitment.current_count, recruitment.max_count)
    }

    fn get_status_color(recruitment: &battle_recruitments::Model) -> u32 {
        match recruitment.status.as_str() {
            "open" => 0x00ff00,      // 緑
            "full" => 0xffff00,      // 黄
            "started" => 0x0000ff,   // 青
            "cancelled" => 0xff0000, // 赤
            _ => 0x808080,           // グレー
        }
    }

    fn format_participants(participants: &[ParticipantInfo]) -> String {
        if participants.is_empty() {
            return "なし".to_string();
        }
        participants
            .iter()
            .map(|p| format!("<@{}>", p.user_id))
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn format_time(time: chrono::NaiveDateTime) -> String {
        time.format("%Y/%m/%d %H:%M").to_string()
    }

    fn format_footer(recruitment: &battle_recruitments::Model) -> String {
        format!("募集ID: {}", recruitment.id)
    }

    fn get_text(key: &str, locale: &str) -> String {
        // i18n対応（簡易版）
        match (key, locale) {
            ("button.join", "ja") => "参加",
            ("button.leave", "ja") => "離脱",
            ("button.cancel", "ja") => "キャンセル",
            ("button.confirm", "ja") => "確定",
            ("button.abort", "ja") => "やめる",
            ("cancel.title", "ja") => "キャンセル確認",
            ("cancel.confirm_message", "ja") => "本当にこの募集をキャンセルしますか？",
            // デフォルト（日本語）
            ("button.join", _) => "参加",
            ("button.leave", _) => "離脱",
            _ => key,
        }
        .to_string()
    }
}
```

### 2. AutoRecruitmentPresenter

自動募集関連の表示を担当。

```rust
// src/presenter/auto_recruitment_presenter.rs

use crate::domain::models::{EmbedData, ComponentRow, SelectMenuData, SelectOption};

/// 自動募集UIの表示を担当するPresenter
pub struct AutoRecruitmentPresenter;

impl AutoRecruitmentPresenter {
    /// クエスト選択メニューを生成
    pub fn create_quest_select_menu(
        quests: &[QuestInfo],
        selected_quest_id: Option<i64>,
        locale: &str,
    ) -> ComponentRow {
        let options: Vec<SelectOption> = quests
            .iter()
            .map(|quest| {
                SelectOption {
                    label: quest.name.clone(),
                    value: quest.id.to_string(),
                    description: Some(quest.description.clone()),
                    emoji: quest.emoji.clone(),
                    default: Some(quest.id) == selected_quest_id,
                }
            })
            .collect();

        let menu = SelectMenuData {
            custom_id: "quest_select".to_string(),
            placeholder: Some(Self::get_text("placeholder.quest", locale)),
            options,
            min_values: Some(1),
            max_values: Some(1),
            disabled: false,
        };

        ComponentRow::select_menu(menu)
    }

    /// 時間選択メニューを生成
    pub fn create_time_select_menu(
        available_times: &[TimeSlot],
        locale: &str,
    ) -> ComponentRow {
        let options: Vec<SelectOption> = available_times
            .iter()
            .map(|slot| {
                SelectOption {
                    label: slot.display_time.clone(),
                    value: slot.value.clone(),
                    description: None,
                    emoji: None,
                    default: false,
                }
            })
            .collect();

        let menu = SelectMenuData {
            custom_id: "time_select".to_string(),
            placeholder: Some(Self::get_text("placeholder.time", locale)),
            options,
            min_values: Some(1),
            max_values: Some(1),
            disabled: false,
        };

        ComponentRow::select_menu(menu)
    }

    /// 設定完了メッセージを生成
    pub fn create_setup_complete_embed(
        settings: &AutoRecruitmentSettings,
        locale: &str,
    ) -> EmbedData {
        EmbedData::new()
            .title(Self::get_text("setup.complete_title", locale))
            .description(Self::get_text("setup.complete_description", locale))
            .color(0x00ff00)
            .field("クエスト", &settings.quest_name, true)
            .field("時刻", &settings.time_display, true)
    }

    fn get_text(key: &str, locale: &str) -> String {
        match (key, locale) {
            ("placeholder.quest", "ja") => "クエストを選択",
            ("placeholder.time", "ja") => "時間を選択",
            ("setup.complete_title", "ja") => "設定完了",
            ("setup.complete_description", "ja") => "自動募集の設定が完了しました",
            _ => key,
        }
        .to_string()
    }
}
```

### 3. NotificationPresenter

通知関連の表示を担当。

```rust
// src/presenter/notification_presenter.rs

use crate::domain::models::EmbedData;

/// 通知の表示を担当するPresenter
pub struct NotificationPresenter;

impl NotificationPresenter {
    /// 募集開始通知Embedを生成
    pub fn create_start_notification(
        recruitment: &RecruitmentInfo,
        mention_users: &[u64],
        locale: &str,
    ) -> (String, EmbedData) {
        let mentions = mention_users
            .iter()
            .map(|id| format!("<@{}>", id))
            .collect::<Vec<_>>()
            .join(" ");

        let embed = EmbedData::new()
            .title("🎮 募集開始")
            .description(format!(
                "「{}」の募集が開始されました！",
                recruitment.quest_name
            ))
            .color(0x00ff00)
            .field("開始時刻", &recruitment.start_time_display, true)
            .field("参加者数", format!("{}", recruitment.participant_count), true);

        (mentions, embed)
    }

    /// 解散通知Embedを生成
    pub fn create_dissolution_embed(
        recruitment: &RecruitmentInfo,
        locale: &str,
    ) -> EmbedData {
        EmbedData::new()
            .title("募集終了")
            .description(format!(
                "「{}」の募集は終了しました",
                recruitment.quest_name
            ))
            .color(0x808080)
    }

    /// マッチング成功通知を生成
    pub fn create_matching_notification(
        matched_users: &[MatchedUserInfo],
        quest_name: &str,
        locale: &str,
    ) -> EmbedData {
        let user_list = matched_users
            .iter()
            .map(|u| format!("• <@{}> ({})", u.user_id, u.quest_status))
            .collect::<Vec<_>>()
            .join("\n");

        EmbedData::new()
            .title("✨ マッチング成功")
            .description(format!("「{}」のマッチングが成立しました！", quest_name))
            .color(0x00ff00)
            .field("参加者", user_list, false)
    }
}
```

### 4. SchedulePresenter

スケジュール表示を担当。

```rust
// src/presenter/schedule_presenter.rs

use crate::domain::models::{EmbedData, AutocompleteOption};

/// スケジュールの表示を担当するPresenter
pub struct SchedulePresenter;

impl SchedulePresenter {
    /// スケジュール一覧Embedを生成
    pub fn create_schedule_list_embed(
        schedules: &[ScheduleInfo],
        locale: &str,
    ) -> EmbedData {
        let mut embed = EmbedData::new()
            .title("📅 スケジュール一覧")
            .color(0x3498db);

        for schedule in schedules {
            embed = embed.field(
                &schedule.quest_name,
                format!("{} - {}", schedule.time_display, schedule.status_display),
                false,
            );
        }

        if schedules.is_empty() {
            embed = embed.description("スケジュールはありません");
        }

        embed
    }

    /// スケジュール選択用オートコンプリートオプションを生成
    pub fn create_schedule_autocomplete(
        schedules: &[ScheduleInfo],
    ) -> Vec<AutocompleteOption> {
        schedules
            .iter()
            .map(|s| {
                AutocompleteOption::new(
                    format!("{} ({})", s.quest_name, s.time_display),
                    s.id.to_string(),
                )
            })
            .collect()
    }
}
```

### 5. AutocompletePresenter

オートコンプリート生成を担当。

```rust
// src/presenter/autocomplete_presenter.rs

use crate::domain::models::AutocompleteOption;

/// オートコンプリートの表示を担当するPresenter
pub struct AutocompletePresenter;

impl AutocompletePresenter {
    /// タイムゾーンオートコンプリートを生成
    pub fn create_timezone_autocomplete(
        timezones: &[TimezoneInfo],
        partial: &str,
    ) -> Vec<AutocompleteOption> {
        timezones
            .iter()
            .filter(|tz| {
                tz.name.to_lowercase().contains(&partial.to_lowercase())
                    || tz.offset.contains(partial)
            })
            .take(25) // Discord制限
            .map(|tz| {
                AutocompleteOption::new(
                    format!("{} ({})", tz.name, tz.offset),
                    tz.id.clone(),
                )
            })
            .collect()
    }

    /// クエストオートコンプリートを生成
    pub fn create_quest_autocomplete(
        quests: &[QuestInfo],
        partial: &str,
    ) -> Vec<AutocompleteOption> {
        quests
            .iter()
            .filter(|q| q.name.to_lowercase().contains(&partial.to_lowercase()))
            .take(25)
            .map(|q| AutocompleteOption::new(&q.name, q.id.to_string()))
            .collect()
    }

    /// バトルスタイルオートコンプリートを生成
    pub fn create_battle_style_autocomplete(
        styles: &[BattleStyleInfo],
        partial: &str,
    ) -> Vec<AutocompleteOption> {
        styles
            .iter()
            .filter(|s| s.name.to_lowercase().contains(&partial.to_lowercase()))
            .take(25)
            .map(|s| AutocompleteOption::new(&s.name, s.id.to_string()))
            .collect()
    }

    /// チャンネルタイプオートコンプリートを生成
    pub fn create_channel_type_autocomplete(
        channel_types: &[ChannelTypeInfo],
        partial: &str,
    ) -> Vec<AutocompleteOption> {
        channel_types
            .iter()
            .filter(|ct| ct.display_name.to_lowercase().contains(&partial.to_lowercase()))
            .take(25)
            .map(|ct| AutocompleteOption::new(&ct.display_name, ct.id.clone()))
            .collect()
    }
}
```

## ディレクトリ構成

```
src/presenter/
├── mod.rs
├── recruitment_presenter.rs       # 募集表示
├── auto_recruitment_presenter.rs  # 自動募集UI
├── notification_presenter.rs      # 通知表示
├── schedule_presenter.rs          # スケジュール表示
└── autocomplete_presenter.rs      # オートコンプリート生成
```

## 移行パターン

### Before: Service層でUI構築

```rust
// src/services/recruitment/new.rs（変更前）
use poise::serenity_prelude::{CreateEmbed, CreateButton, ButtonStyle};

impl RecruitmentService {
    pub fn create_recruitment_message(&self, data: &RecruitmentData) -> CreateEmbed {
        CreateEmbed::new()
            .title(&data.quest_name)
            .description(format!("参加者: {}/{}", data.current, data.max))
            .color(0x00ff00)
    }
}
```

### After: Presenter層でUI構築

```rust
// src/services/recruitment/new.rs（変更後）
// UIビルダーへの依存なし
impl RecruitmentService {
    pub fn get_recruitment_data(&self, id: i64) -> Result<RecruitmentData, Error> {
        // 純粋なビジネスロジックのみ
        let recruitment = self.repository.find_by_id(id)?;
        let participants = self.repository.get_participants(id)?;
        Ok(RecruitmentData {
            recruitment,
            participants,
        })
    }
}

// src/presenter/recruitment_presenter.rs（新規）
impl RecruitmentPresenter {
    pub fn create_recruitment_embed(data: &RecruitmentData, locale: &str) -> EmbedData {
        EmbedData::new()
            .title(&data.recruitment.quest_name)
            .description(format!("参加者: {}/{}", data.current, data.max))
            .color(0x00ff00)
    }
}

// src/events/commands/recruitment.rs（呼び出し側）
async fn show_recruitment(ctx: Context<'_>, id: i64) -> Result<(), Error> {
    let data = service.get_recruitment_data(id)?;
    let embed = RecruitmentPresenter::create_recruitment_embed(&data, ctx.locale());
    // Gateway経由で送信
}
```

### Before: Service層でAutocompleteChoice返却

```rust
// src/services/timezone_service.rs（変更前）
use poise::serenity_prelude::AutocompleteChoice;

impl TimezoneService {
    pub fn get_timezones_for_autocomplete(partial: &str) -> Vec<AutocompleteChoice> {
        TIMEZONES
            .iter()
            .filter(|tz| tz.name.contains(partial))
            .map(|tz| AutocompleteChoice::new(tz.name.clone(), tz.id.clone()))
            .collect()
    }
}
```

### After: Service + Presenterで分離

```rust
// src/services/timezone_service.rs（変更後）
impl TimezoneService {
    pub fn search_timezones(partial: &str) -> Vec<TimezoneInfo> {
        // 純粋な検索ロジックのみ
        TIMEZONES
            .iter()
            .filter(|tz| tz.name.to_lowercase().contains(&partial.to_lowercase()))
            .cloned()
            .collect()
    }
}

// src/presenter/autocomplete_presenter.rs（新規）
impl AutocompletePresenter {
    pub fn create_timezone_autocomplete(
        timezones: &[TimezoneInfo],
        partial: &str,
    ) -> Vec<AutocompleteOption> {
        // 表示フォーマットのみ担当
        timezones
            .iter()
            .take(25)
            .map(|tz| AutocompleteOption::new(
                format!("{} ({})", tz.name, tz.offset),
                tz.id.clone(),
            ))
            .collect()
    }
}
```

## 完了条件

- [ ] RecruitmentPresenterが実装されている
- [ ] AutoRecruitmentPresenterが実装されている
- [ ] NotificationPresenterが実装されている
- [ ] SchedulePresenterが実装されている
- [ ] AutocompletePresenterが実装されている
- [ ] Service層からUIビルダー依存が除去されている
- [ ] Facade層からUIビルダー依存が除去されている

## 注意事項

1. **Presenterは状態を持たない** - 純粋な変換関数のみ
2. **ロケール対応はPresenter層で行う** - Service層はロケール非依存
3. **ドメインモデル（EmbedData等）を返す** - serenity型は返さない
4. **Gateway層でserenity型への最終変換を行う**
