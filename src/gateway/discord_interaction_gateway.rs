//! Discordインタラクション操作Gatewayトレイト

use async_trait::async_trait;

use crate::errors::GatewayError;
use crate::types::discord::{DiscordInteractionId, InteractionResponse};

/// Discordインタラクション操作を抽象化するトレイト
///
/// ボタン、セレクトメニュー等のコンポーネントインタラクションへの応答を提供する。
/// ビジネスロジック層はこのトレイトを通じてインタラクションを処理する。
#[async_trait]
pub trait DiscordInteractionGateway: Send + Sync {
    /// インタラクションを遅延応答する
    ///
    /// インタラクションに対して「処理中」の応答を返す。
    /// これにより、3秒以上かかる処理でもタイムアウトしない。
    ///
    /// # Arguments
    ///
    /// * `interaction_id` - インタラクションID
    /// * `interaction_token` - インタラクショントークン
    async fn defer_interaction(
        &self,
        interaction_id: DiscordInteractionId,
        interaction_token: &str,
    ) -> Result<(), GatewayError>;

    /// インタラクションに応答する
    ///
    /// # Arguments
    ///
    /// * `interaction_id` - インタラクションID
    /// * `interaction_token` - インタラクショントークン
    /// * `response` - 応答内容
    async fn respond_to_interaction(
        &self,
        interaction_id: DiscordInteractionId,
        interaction_token: &str,
        response: InteractionResponse,
    ) -> Result<(), GatewayError>;

    /// インタラクション応答を編集する
    ///
    /// 既に送信した応答を編集する。
    ///
    /// # Arguments
    ///
    /// * `interaction_id` - インタラクションID
    /// * `interaction_token` - インタラクショントークン
    /// * `response` - 新しい応答内容
    async fn edit_interaction_response(
        &self,
        interaction_id: DiscordInteractionId,
        interaction_token: &str,
        response: InteractionResponse,
    ) -> Result<(), GatewayError>;
}
