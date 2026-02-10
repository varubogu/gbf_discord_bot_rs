//! イベントレイヤー用ヘルパー関数
//!
//! PoiseContextを使用する処理を集約。servicesレイヤーに依存しつつ、
//! poise依存をeventsレイヤーに閉じ込める。

use crate::repository::database::guild_message_text_repository::SeaOrmGuildMessageTextRepository;
use crate::repository::database::message_text_repository::SeaOrmMessageTextRepository;
use crate::services::message::{MessageService, MessageTextId};
use crate::types::PoiseContext;
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

/// PoiseContextから最適なロケールを取得
///
/// 優先順位: ユーザーロケール → ギルドロケール → None（デフォルトはmessage_serviceで"en"にフォールバック）
pub fn get_locale_from_context(ctx: &PoiseContext<'_>) -> Option<String> {
    // Poiseのコンテキストからユーザーまたはギルドのロケールを取得
    // Discord APIから返される locale() を使用
    ctx.locale().map(|s| s.to_string())
}

/// PoiseContextからギルドIDを取得
pub fn get_guild_id_from_context(ctx: &PoiseContext<'_>) -> Option<i64> {
    ctx.guild_id().map(|id| id.get() as i64)
}

/// PoiseContextを使用してメッセージを取得
///
/// # 引数
/// * `ctx` - Poiseコンテキスト
/// * `message_service` - メッセージサービス
/// * `message_id` - メッセージID（MessageId enum または &str）
/// * `params` - パラメータ
pub async fn get_message_from_context<T: IntoMessageId>(
    ctx: &PoiseContext<'_>,
    message_service: &MessageService<SeaOrmGuildMessageTextRepository, SeaOrmMessageTextRepository>,
    message_id: T,
    params: HashMap<String, String>,
) -> Result<String, crate::errors::ServiceError> {
    let guild_id = get_guild_id_from_context(ctx);
    let locale = get_locale_from_context(ctx);
    let message_id_str = message_id.into_message_id();

    message_service
        .get_message(
            ctx.data().app_state.guild_db(),
            &message_id_str,
            params,
            guild_id,
            locale.as_deref(),
        )
        .await
}
