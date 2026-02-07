/// メッセージサービス用ヘルパー関数
use crate::repository::{GuildMessageTextRepository, MessageTextRepository};
use crate::services::message::{MessageService, MessageTextId};
use sea_orm::DatabaseConnection;
use std::collections::HashMap;

/// メッセージIDとして使用できる型のトレイト
pub trait IntoMessageId {
    fn into_message_id(self) -> String;
}

impl IntoMessageId for MessageTextId {
    fn into_message_id(self) -> String {
        self.as_str().to_string()
    }
}

impl IntoMessageId for &str {
    fn into_message_id(self) -> String {
        self.to_string()
    }
}

impl IntoMessageId for String {
    fn into_message_id(self) -> String {
        self
    }
}

/// 指定されたコンテキスト情報を使用してメッセージを取得
///
/// # 引数
/// * `db` - データベース接続
/// * `message_service` - メッセージサービス
/// * `message_id` - メッセージID（MessageId enum または &str）
/// * `params` - パラメータ
/// * `guild_id` - ギルドID（省略可能）
/// * `locale` - ロケール（省略可能）
pub async fn get_message<
    T: IntoMessageId,
    GM: GuildMessageTextRepository,
    MT: MessageTextRepository,
>(
    db: &DatabaseConnection,
    message_service: &MessageService<GM, MT>,
    message_id: T,
    params: HashMap<String, String>,
    guild_id: Option<i64>,
    locale: Option<&str>,
) -> Result<String, crate::errors::ServiceError> {
    let message_id_str = message_id.into_message_id();

    message_service
        .get_message(db, &message_id_str, params, guild_id, locale)
        .await
}
