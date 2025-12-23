/// メッセージサービス用ヘルパー関数
use crate::services::message::MessageService;
use crate::types::PoiseContext;
use std::collections::HashMap;

/// PoiseContextから最適なロケールを取得
///
/// 優先順位: ユーザーロケール → ギルドロケール → "en"
pub fn get_locale_from_context(_ctx: &PoiseContext<'_>) -> Option<String> {
    // Poiseの場合、ctx.locale()でロケールが取得できる可能性がある
    // 取得できない場合はNoneを返す
    // TODO: Poiseのバージョンによって異なる可能性があるため、実際の動作で確認が必要
    None
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
/// * `message_id` - メッセージID
/// * `params` - パラメータ
/// * `yaml_key` - YAMLキー
pub async fn get_message_from_context(
    ctx: &PoiseContext<'_>,
    message_service: &MessageService,
    message_id: &str,
    params: HashMap<String, String>,
    yaml_key: &str,
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
            yaml_key,
        )
        .await
}
