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
