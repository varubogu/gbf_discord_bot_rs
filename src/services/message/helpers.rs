/// メッセージサービス用ヘルパー関数
use crate::services::message::MessageService;
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
/// * `message_id` - メッセージID（DB検索キー兼YAMLキー）
/// * `params` - パラメータ
pub async fn get_message_from_context(
    ctx: &PoiseContext<'_>,
    message_service: &MessageService,
    message_id: &str,
    params: HashMap<String, String>,
) -> Result<String, crate::errors::ServiceError> {
    let guild_id = get_guild_id_from_context(ctx);
    let locale = get_locale_from_context(ctx);

    message_service
        .get_message(
            ctx.data().app_state.guild_db(),
            message_id,
            params,
            guild_id,
            locale.as_deref(),
        )
        .await
}
