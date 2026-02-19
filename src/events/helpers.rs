//! イベントレイヤー用ヘルパー関数
//!
//! PoiseContextを使用する処理を集約。servicesレイヤーに依存しつつ、
//! poise依存をeventsレイヤーに閉じ込める。

use crate::repository::{GuildMessageTextRepository, MessageTextRepository};
use crate::services::message::{MessageService, MessageTextId};
use crate::types::PoiseContext;
use std::collections::HashMap;

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
/// * `message_id` - メッセージID（`MessageTextId`）
/// * `params` - パラメータ
pub async fn get_message_from_context<G, M>(
    ctx: &PoiseContext<'_>,
    message_service: &MessageService<G, M>,
    message_id: MessageTextId,
    params: HashMap<String, String>,
) -> Result<String, crate::errors::ServiceError>
where
    G: GuildMessageTextRepository,
    M: MessageTextRepository,
{
    let guild_id = get_guild_id_from_context(ctx);
    let locale = get_locale_from_context(ctx);

    message_service
        .get_message(
            ctx.data().app_state.guild_db(),
            message_id.as_str(),
            params,
            guild_id,
            locale.as_deref(),
        )
        .await
}

/// メッセージ取得に失敗した場合、呼び出し側で指定した文言へフォールバックする
pub async fn get_message_or_fallback_from_context<G, M>(
    ctx: &PoiseContext<'_>,
    message_service: &MessageService<G, M>,
    message_id: MessageTextId,
    params: HashMap<String, String>,
    fallback_text: &str,
) -> String
where
    G: GuildMessageTextRepository,
    M: MessageTextRepository,
{
    get_message_from_context(ctx, message_service, message_id, params)
        .await
        .unwrap_or_else(|_| fallback_text.to_string())
}
