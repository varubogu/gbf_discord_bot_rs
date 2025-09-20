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
}
