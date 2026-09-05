use chrono::{DateTime, Utc};

/// キャンセル可能性チェックの結果
#[derive(Debug, PartialEq)]
pub enum CanCancelResult {
    /// キャンセル可能
    Success,
    /// 既にキャンセル済み
    AlreadyCancelled,
    /// 募集メッセージは過去にあったが、削除済み
    MessageDeleted,
    /// 募集メッセージじゃない
    NotRecruitMessage,
    /// 募集がなく、メッセージもない
    NotFound,
    /// 開催日時を過ぎているためキャンセル対象外
    EventDatePassed,
    /// 操作権限なし（募集主本人でも gbf_bot_control ロール保持者でもない）
    PermissionDenied,
}

/// メッセージ削除時のキャンセル処理結果
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CancelOnDeleteResult {
    /// キャンセル処理完了
    Cancelled,
    /// 募集メッセージではない
    NotRecruitmentMessage,
    /// 既にキャンセル済み
    AlreadyCancelled,
    /// 開催日時を過ぎているためキャンセル対象外
    EventDatePassed,
}

/// 出発日時の後ろ倒し判定結果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostponeDepartureResult {
    /// 後ろ倒し可能（変更後の出発日時）
    Postponed(DateTime<Utc>),
    /// 既に出発時刻を過ぎているため後ろ倒し対象外
    EventDatePassed,
}
