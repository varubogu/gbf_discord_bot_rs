use crate::models::message_texts::MessageTexts;
use async_trait::async_trait;
use sea_orm::DbErr;

/// メッセージテキストリポジトリの抽象インターフェース
/// データベースアクセスの詳細を隠蔽し、「データを保存する何か」への依存のみ提供
#[async_trait]
pub trait MessageTextRepository: Send + Sync {
    /// メッセージIDでメッセージテキストを取得
    async fn get_by_id<'c, C>(
        &self,
        db: &'c C,
        id: &str,
    ) -> Result<Option<MessageTexts>, DbErr>
    where
        C: sea_orm::ConnectionTrait;
}
