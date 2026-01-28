//! プレゼンテーション層
//!
//! ドメインモデルをUI表示用モデルに変換する責務を担う。
//! Service層からUIビルダー依存を除去するために使用する。
//!
//! # 責務
//!
//! - ドメインモデル → UI表示用モデル（`EmbedContent`, `ActionRowContent`等）への変換
//! - 表示フォーマットの決定（日付フォーマット、色、レイアウト）
//! - i18n/l10n対応（ロケールに応じたテキスト生成）
//! - UIコンポーネントの組み立て
//!
//! # 注意事項
//!
//! - Presenterは状態を持たない（純粋な変換関数のみ）
//! - ロケール対応はPresenter層で行う（Service層はロケール非依存）
//! - ドメインモデル（`EmbedContent`等）を返す（serenity型は返さない）
//! - Gateway層でserenity型への最終変換を行う
//!
//! # 構成
//!
//! - `AutocompletePresenter` - オートコンプリート候補の生成
//! - `RecruitmentPresenter` - 募集表示（Embed、ボタン、セレクトメニュー）
//! - `AutoRecruitmentPresenter` - 自動募集UI
//! - `NotificationPresenter` - 各種通知メッセージ
//! - `SchedulePresenter` - 定期募集スケジュール表示

pub mod auto_recruitment_presenter;
pub mod autocomplete_presenter;
pub mod notification_presenter;
pub mod recruitment_presenter;
pub mod schedule_presenter;

pub use auto_recruitment_presenter::{AutoRecruitmentPresenter, ElementInfo, get_six_elements};
pub use autocomplete_presenter::AutocompletePresenter;
pub use notification_presenter::NotificationPresenter;
pub use recruitment_presenter::RecruitmentPresenter;
pub use schedule_presenter::{ScheduleCreationDisplayInfo, ScheduleDisplayInfo, SchedulePresenter};
