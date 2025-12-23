use crate::facades::timezone::TimezoneFacade;
use crate::services::message::helpers::get_message_from_context;
use crate::services::message::MessageId;
use crate::types::{PoiseContext, Result};
use std::collections::HashMap;
use std::sync::Arc;

#[poise::command(
    slash_command,
    guild_only,
    ephemeral = true,
    name_localized("ja", "タイムゾーン確認"),
    description_localized("ja", "サーバーの現在のタイムゾーン設定を確認します")
)]
pub async fn timezone_show(ctx: PoiseContext<'_>) -> Result<()> {
    ctx.defer_ephemeral().await?;

    // ギルドIDを取得
    let guild_id = ctx.guild_id().ok_or_else(|| {
        crate::types::AppError::Generic("このコマンドはサーバー内でのみ使用できます".to_string())
    })?;

    // タイムゾーンを取得（Facade経由）
    let app_state = &ctx.data().app_state;
    let facade = TimezoneFacade::new(Arc::new(app_state.clone()));
    let timezone = facade.get_timezone(guild_id.get() as i64).await?;

    // 結果メッセージ（デフォルト判定はスコープ外のため省略）
    let mut params = HashMap::new();
    params.insert("timezone".to_string(), timezone.name().to_string());

    let message = get_message_from_context(
        &ctx,
        ctx.data().app_state.message_service(),
        MessageId::TimezoneShowCurrent,
        params,
    )
    .await
    .unwrap_or_else(|_| format!("現在のタイムゾーン: {}", timezone.name()));

    ctx.say(&message).await?;

    Ok(())
}
