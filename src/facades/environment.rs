use crate::types::{AppError, Result};
use tracing::warn;

/// 環境変数読み込み処理
///
/// # 引数
/// - `_guild_id`: ギルドID
///
/// # 戻り値
/// - `Result<()>`: 処理結果
///
/// # Note
/// この機能は未実装です
pub(crate) async fn load(guild_id: u64) -> Result<()> {
    warn!(
        guild_id,
        "設定値リロード機能は未実装のため、明示的エラーを返します"
    );
    Err(AppError::Business {
        message: "設定値リロード機能は現在利用できません。".to_string(),
    })
}
