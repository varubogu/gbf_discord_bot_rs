use crate::models::message_texts::MessageTexts;
use crate::types::Result;
use async_trait::async_trait;

/// メッセージテキストリポジトリの抽象インターフェース
/// データベースアクセスの詳細を隠蔽し、「データを保存する何か」への依存のみ提供
#[async_trait]
pub trait MessageTextRepository: Send + Sync {
    /// メッセージIDでメッセージテキストを取得
    async fn get_by_message(&self, message_id: &str) -> Result<Option<MessageTexts>>;
}
