use thiserror::Error;

/// 募集系の入力・操作エラー
#[derive(Debug, Error)]
pub enum RecruitmentError {
    #[error("このコマンドはサーバー内でのみ使用できます")]
    GuildOnly,

    #[error("不正なカスタムIDです")]
    InvalidCustomId,

    #[error("{field}の解析に失敗しました")]
    ParseFailed { field: &'static str },

    #[error("{field}が入力されていません")]
    MissingInput { field: &'static str },

    #[error("{field}が選択されていません")]
    NotSelected { field: &'static str },

    #[error("{field}が見つかりません")]
    NotFound { field: &'static str },

    #[error("予期しないコンポーネントタイプです")]
    UnexpectedComponentType,

    #[error("{message}")]
    Message { message: String },
}
