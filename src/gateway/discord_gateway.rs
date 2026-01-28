//! 統合Discord Gatewayトレイト

use super::{
    DiscordChannelGateway, DiscordGuildGateway, DiscordInteractionGateway, DiscordMessageGateway,
    DiscordReactionGateway,
};

/// すべてのDiscord Gateway機能を統合したトレイト
///
/// 複数のGatewayメソッドを使用するServiceやFacadeで、
/// 型パラメータを減らすために使用する。
///
/// # Example
///
/// ```ignore
/// use crate::gateway::DiscordGateway;
///
/// struct MyService<G: DiscordGateway> {
///     gateway: Arc<G>,
/// }
///
/// impl<G: DiscordGateway> MyService<G> {
///     async fn do_something(&self) {
///         // メッセージ送信（DiscordMessageGateway）
///         self.gateway.send_message(...).await?;
///
///         // リアクション追加（DiscordReactionGateway）
///         self.gateway.add_reaction(...).await?;
///     }
/// }
/// ```
pub trait DiscordGateway:
    DiscordMessageGateway
    + DiscordChannelGateway
    + DiscordInteractionGateway
    + DiscordReactionGateway
    + DiscordGuildGateway
{
}

/// 全てのGatewayトレイトを実装している型には自動的にDiscordGatewayを実装
impl<T> DiscordGateway for T where
    T: DiscordMessageGateway
        + DiscordChannelGateway
        + DiscordInteractionGateway
        + DiscordReactionGateway
        + DiscordGuildGateway
{
}
