use crate::gateway::DiscordGateway;
use crate::types::Result;
use async_trait::async_trait;
use std::sync::Arc;

/// スケジューラー実行ユースケース
///
/// トランザクション境界は実装側（Facade）で管理する。
#[async_trait]
pub trait SchedulerDispatchUseCase<G>: Send + Sync + 'static
where
    G: DiscordGateway + Send + Sync + 'static,
{
    /// 起動時補正処理を実行する
    async fn repair_on_startup(&self, gateway: &Arc<G>) -> Result<()>;

    /// 定期実行サイクルを1回実行する
    async fn dispatch_due_tasks(&self, gateway: &Arc<G>) -> Result<()>;
}
