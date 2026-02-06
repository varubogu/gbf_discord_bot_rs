# Step 4: 依存性注入の改善

## 目的

`Arc<Http>`や`Context`がService/Facade層に直接渡されている現状を改善し、Gateway抽象化を通じた適切な依存性注入パターンを確立する。

## 概要

```
┌─────────────────────────────────────────────────────────────┐
│                     main.rs / Bot Setup                      │
│         (DI Container: Gateway, Service, Facadeの構築)       │
└─────────────────────────────────────────────────────────────┘
                              │
           ┌──────────────────┼──────────────────┐
           ▼                  ▼                  ▼
    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
    │   Gateway   │    │   Service   │    │   Facade    │
    │ (Arc<Http>) │◄───│  (Arc<G>)   │◄───│ (Arc<Svc>)  │
    └─────────────┘    └─────────────┘    └─────────────┘
```

## 静的ディスパッチによるDI設計

本計画では**静的ディスパッチ（ジェネリクス）**を採用する。コンテナもジェネリクスで定義し、本番用とテスト用で異なる具象型を使用する。

## DIコンテナの設計

### 1. Servicesコンテナ（ジェネリクス版）

```rust
// src/di/services.rs

use std::sync::Arc;
use crate::gateway::DiscordGateway;
use crate::repository::Repositories;
use crate::services::*;

/// Service群を保持するコンテナ（静的ディスパッチ版）
#[derive(Clone)]
pub struct Services<G>
where
    G: DiscordGateway,
{
    pub recruitment: Arc<RecruitmentService<G>>,
    pub notification: Arc<NotificationService<G>>,
    pub permission: Arc<PermissionService<G>>,
    pub scheduler: Arc<SchedulerManager<G>>,
    pub guild_environment: Arc<GuildEnvironmentService<G>>,
    pub timezone: Arc<TimezoneService>,
    pub channel: Arc<ChannelService<G>>,
}

impl<G> Services<G>
where
    G: DiscordGateway + Clone + 'static,
{
    pub fn new(
        gateway: Arc<G>,
        repositories: Repositories,
    ) -> Self {
        let recruitment = Arc::new(RecruitmentService::new(
            gateway.clone(),
            repositories.battle_recruitments.clone(),
        ));

        let notification = Arc::new(NotificationService::new(
            gateway.clone(),
        ));

        let permission = Arc::new(PermissionService::new(
            gateway.clone(),
        ));

        let scheduler = Arc::new(SchedulerManager::new(
            gateway.clone(),
            repositories.clone(),
        ));

        let guild_environment = Arc::new(GuildEnvironmentService::new(
            gateway.clone(),
        ));

        let timezone = Arc::new(TimezoneService::new());

        let channel = Arc::new(ChannelService::new(
            gateway.clone(),
            repositories.channels.clone(),
        ));

        Self {
            recruitment,
            notification,
            permission,
            scheduler,
            guild_environment,
            timezone,
            channel,
        }
    }
}

// 本番用型エイリアス
pub type ProductionServices = Services<PoiseDiscordGateway>;
```

### 2. Facadesコンテナ（ジェネリクス版）

```rust
// src/di/facades.rs

use std::sync::Arc;
use crate::gateway::DiscordGateway;
use crate::di::Services;
use crate::facades::*;

/// Facade群を保持するコンテナ（静的ディスパッチ版）
#[derive(Clone)]
pub struct Facades<G>
where
    G: DiscordGateway,
{
    pub recruitment: Arc<RecruitmentFacade<G>>,
    pub auto_recruitment: Arc<AutoRecruitmentFacade<G>>,
    pub guild_settings: Arc<GuildSettingsFacade>,
    pub channel_management: Arc<ChannelManagementFacade<G>>,
}

impl<G> Facades<G>
where
    G: DiscordGateway + Clone + 'static,
{
    pub fn new(services: Services<G>, gateway: Arc<G>) -> Self {
        let recruitment = Arc::new(RecruitmentFacade::new(
            services.recruitment.clone(),
            services.notification.clone(),
            gateway.clone(),
        ));

        let auto_recruitment = Arc::new(AutoRecruitmentFacade::new(
            services.recruitment.clone(),
            services.scheduler.clone(),
            gateway.clone(),
        ));

        let guild_settings = Arc::new(GuildSettingsFacade::new(
            services.timezone.clone(),
        ));

        let channel_management = Arc::new(ChannelManagementFacade::new(
            services.channel.clone(),
        ));

        Self {
            recruitment,
            auto_recruitment,
            guild_settings,
            channel_management,
        }
    }
}

// 本番用型エイリアス
pub type ProductionFacades = Facades<PoiseDiscordGateway>;
```

### 3. アプリケーションコンテナ（ジェネリクス版）

```rust
// src/di/container.rs

use std::sync::Arc;
use sea_orm::DatabaseConnection;
use crate::gateway::DiscordGateway;
use crate::gateway::impl_poise::PoiseDiscordGateway;
use crate::di::{Services, Facades, Repositories};

/// アプリケーション全体のDIコンテナ（静的ディスパッチ版）
#[derive(Clone)]
pub struct AppContainer<G>
where
    G: DiscordGateway,
{
    pub gateway: Arc<G>,
    pub repositories: Repositories,
    pub services: Services<G>,
    pub facades: Facades<G>,
}

impl<G> AppContainer<G>
where
    G: DiscordGateway + Clone + 'static,
{
    /// 汎用コンストラクタ
    pub fn new(gateway: Arc<G>, db: Arc<DatabaseConnection>) -> Self {
        let repositories = Repositories::new(db);
        let services = Services::new(gateway.clone(), repositories.clone());
        let facades = Facades::new(services.clone(), gateway.clone());

        Self {
            gateway,
            repositories,
            services,
            facades,
        }
    }
}

// 本番環境用の専用実装
impl AppContainer<PoiseDiscordGateway> {
    /// 本番環境用コンテナを構築
    pub fn new_production(
        http: Arc<poise::serenity_prelude::Http>,
        db: Arc<DatabaseConnection>,
    ) -> Self {
        let gateway = Arc::new(PoiseDiscordGateway::new(http));
        Self::new(gateway, db)
    }
}

// 本番用型エイリアス
pub type ProductionAppContainer = AppContainer<PoiseDiscordGateway>;

// テスト用型エイリアス
#[cfg(test)]
pub type TestAppContainer = AppContainer<MockDiscordGateway>;
```

### 4. テスト用コンテナ構築

```rust
// src/di/container.rs（続き）

#[cfg(test)]
use crate::gateway::mock::MockDiscordGateway;

#[cfg(test)]
impl AppContainer<MockDiscordGateway> {
    /// テスト用コンテナを構築
    pub fn new_test(db: Arc<DatabaseConnection>) -> (Self, Arc<MockDiscordGateway>) {
        let gateway = Arc::new(MockDiscordGateway::new());
        let container = Self::new(gateway.clone(), db);
        (container, gateway)
    }

    /// モックGatewayを指定してテスト用コンテナを構築
    pub fn new_test_with_gateway(
        gateway: Arc<MockDiscordGateway>,
        db: Arc<DatabaseConnection>,
    ) -> Self {
        Self::new(gateway, db)
    }
}
```

## 移行パターン

### パターン1: Arc<Http>をGatewayに置換（静的ディスパッチ版）

#### Before

```rust
// src/services/schedule/notification_service.rs（変更前）
use poise::serenity_prelude::{Http, ChannelId, CreateMessage};

pub struct NotificationService {
    http: Arc<Http>,
}

impl NotificationService {
    pub fn new(http: Arc<Http>) -> Self {
        Self { http }
    }

    pub async fn send_notification(&self, channel_id: u64, text: &str) -> Result<(), Error> {
        let channel = ChannelId::new(channel_id);
        let message = CreateMessage::new().content(text);
        channel.send_message(&self.http, message).await?;
        Ok(())
    }
}
```

#### After

```rust
// src/services/schedule/notification_service.rs（変更後）
use crate::gateway::DiscordMessageGateway;
use crate::domain::types::DiscordChannelId;
use crate::domain::models::message::MessageContent;

/// 通知サービス（静的ディスパッチ版）
pub struct NotificationService<G>
where
    G: DiscordMessageGateway,
{
    message_gateway: Arc<G>,
}

impl<G> NotificationService<G>
where
    G: DiscordMessageGateway,
{
    pub fn new(message_gateway: Arc<G>) -> Self {
        Self { message_gateway }
    }

    pub async fn send_notification(
        &self,
        channel_id: DiscordChannelId,
        text: &str,
    ) -> Result<(), Error> {
        let content = MessageContent::text(text);
        self.message_gateway.send_message(channel_id, content).await?;
        Ok(())
    }
}

// 本番用型エイリアス
pub type ProductionNotificationService = NotificationService<PoiseDiscordGateway>;
```

### パターン2: ContextパラメータをRequestContextに抽象化

#### Before

```rust
// src/facades/recruitment/cancel.rs（変更前）
use poise::serenity_prelude::Context;

pub async fn cancel_recruitment(
    ctx: Context<'_, Data, Error>,
    recruitment_id: i64,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let user_id = ctx.author().id;
    let http = ctx.http();

    // Contextを使った処理...
}
```

#### After

```rust
// src/domain/models/request_context.rs（新規）
use crate::domain::types::*;

/// リクエストコンテキスト（Discord Context抽象化）
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub guild_id: DiscordGuildId,
    pub channel_id: DiscordChannelId,
    pub user_id: DiscordUserId,
    pub locale: String,
}

impl RequestContext {
    pub fn new(
        guild_id: DiscordGuildId,
        channel_id: DiscordChannelId,
        user_id: DiscordUserId,
        locale: impl Into<String>,
    ) -> Self {
        Self {
            guild_id,
            channel_id,
            user_id,
            locale: locale.into(),
        }
    }
}

// src/events/commands/recruitment.rs（イベント層での変換）
use poise::serenity_prelude::Context as PoiseContext;

fn extract_request_context(ctx: &PoiseContext<'_, Data, Error>) -> RequestContext {
    RequestContext::new(
        DiscordGuildId(ctx.guild_id().unwrap().get()),
        DiscordChannelId(ctx.channel_id().get()),
        DiscordUserId(ctx.author().id.get()),
        ctx.locale().unwrap_or("ja"),
    )
}

// src/facades/recruitment/cancel.rs（変更後）
use crate::domain::models::RequestContext;
use crate::gateway::DiscordGateway;

pub struct RecruitmentCancelFacade<G>
where
    G: DiscordGateway,
{
    gateway: Arc<G>,
    // ...
}

impl<G> RecruitmentCancelFacade<G>
where
    G: DiscordGateway,
{
    pub async fn cancel_recruitment(
        &self,
        req_ctx: RequestContext,
        recruitment_id: i64,
    ) -> Result<(), Error> {
        // Contextへの依存なし
        let guild_id = req_ctx.guild_id;
        let user_id = req_ctx.user_id;

        // Gateway経由で処理...
    }
}
```

### パターン3: タスクエグゼキューターのDI（静的ディスパッチ版）

#### Before

```rust
// src/services/schedule/dismissal_task_executor.rs（変更前）
use poise::serenity_prelude::{Http, ChannelId, MessageId, EditMessage};

pub struct DismissalTaskExecutor;

impl DismissalTaskExecutor {
    pub async fn execute(
        http: Arc<Http>,  // HttpがパラメータとしてExecutorに渡される
        channel_id: u64,
        message_id: u64,
    ) -> Result<(), Error> {
        let channel = ChannelId::new(channel_id);
        let msg_id = MessageId::new(message_id);
        let message = channel.message(&http, msg_id).await?;

        let edit = EditMessage::new().content("終了しました");
        message.edit(&http, edit).await?;

        Ok(())
    }
}
```

#### After

```rust
// src/services/schedule/dismissal_task_executor.rs（変更後）
use crate::gateway::DiscordMessageGateway;
use crate::domain::types::{DiscordChannelId, DiscordMessageId};
use crate::domain::models::message::MessageContent;

/// 解散タスク実行者（静的ディスパッチ版）
pub struct DismissalTaskExecutor<G>
where
    G: DiscordMessageGateway,
{
    message_gateway: Arc<G>,
}

impl<G> DismissalTaskExecutor<G>
where
    G: DiscordMessageGateway,
{
    pub fn new(message_gateway: Arc<G>) -> Self {
        Self { message_gateway }
    }

    pub async fn execute(
        &self,
        channel_id: DiscordChannelId,
        message_id: DiscordMessageId,
    ) -> Result<(), Error> {
        let content = MessageContent::text("終了しました");
        self.message_gateway
            .edit_message(channel_id, message_id, content)
            .await?;

        Ok(())
    }
}

// 本番用型エイリアス
pub type ProductionDismissalTaskExecutor = DismissalTaskExecutor<PoiseDiscordGateway>;
```

### パターン4: SchedulerManagerのDI改善（静的ディスパッチ版）

#### Before

```rust
// src/services/schedule/scheduler_manager.rs（変更前）
pub struct SchedulerManager {
    http: Arc<Http>,
    // ...
}

impl SchedulerManager {
    pub fn new(http: Arc<Http>) -> Self {
        Self { http }
    }

    pub async fn schedule_dismissal(&self, ...) {
        // 内部でhttpを使ってタスクを実行
        DismissalTaskExecutor::execute(self.http.clone(), ...).await
    }
}
```

#### After

```rust
// src/services/schedule/scheduler_manager.rs（変更後）
use crate::gateway::DiscordGateway;

/// スケジューラーマネージャー（静的ディスパッチ版）
pub struct SchedulerManager<G>
where
    G: DiscordGateway,
{
    dismissal_executor: Arc<DismissalTaskExecutor<G>>,
    dissolution_executor: Arc<DissolutionTaskExecutor<G>>,
    matching_executor: Arc<AutoMatchingTaskExecutor<G>>,
    rotation_executor: Arc<AutoRecruitmentRotationTaskExecutor<G>>,
}

impl<G> SchedulerManager<G>
where
    G: DiscordGateway + Clone + 'static,
{
    pub fn new(
        gateway: Arc<G>,
        repositories: Repositories,
    ) -> Self {
        Self {
            dismissal_executor: Arc::new(DismissalTaskExecutor::new(
                gateway.clone(),
            )),
            dissolution_executor: Arc::new(DissolutionTaskExecutor::new(
                gateway.clone(),
            )),
            matching_executor: Arc::new(AutoMatchingTaskExecutor::new(
                gateway.clone(),
                repositories.clone(),
            )),
            rotation_executor: Arc::new(AutoRecruitmentRotationTaskExecutor::new(
                gateway.clone(),
            )),
        }
    }

    pub async fn schedule_dismissal(&self, ...) {
        // Executorはすでに依存を持っている
        self.dismissal_executor.execute(...).await
    }
}

// 本番用型エイリアス
pub type ProductionSchedulerManager = SchedulerManager<PoiseDiscordGateway>;
```

## main.rsでの初期化

```rust
// src/main.rs

use crate::di::{ProductionAppContainer, AppContainer};
use crate::gateway::impl_poise::PoiseDiscordGateway;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // 環境設定読み込み
    let config = Config::from_env()?;

    // データベース接続
    let db = Arc::new(Database::connect(&config.database_url).await?);

    // Botフレームワーク設定
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![...],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                // DIコンテナ構築（本番用：具象型を使用）
                let http = Arc::new(ctx.http.clone());
                let container = AppContainer::new_production(http, db);

                // Dataとして設定
                Ok(Data {
                    container,
                })
            })
        })
        .build();

    // Bot起動
    let client = serenity::ClientBuilder::new(&config.discord_token, intents)
        .framework(framework)
        .await?;

    client.start().await?;

    Ok(())
}
```

## ディレクトリ構成

```
src/di/
├── mod.rs
├── container.rs      # AppContainer<G>
├── services.rs       # Services<G>
├── facades.rs        # Facades<G>
└── repositories.rs   # Repositories（Gatewayに依存しない）
```

## テストでの使用

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::di::AppContainer;
    use crate::gateway::mock::MockDiscordGateway;

    #[tokio::test]
    async fn test_notification_service() {
        // テスト用DBセットアップ
        let db = setup_test_db().await;

        // モックGateway作成
        let mut mock_gateway = MockDiscordGateway::new();

        // 期待値設定（モックに直接設定）
        mock_gateway
            .expect_send_message()
            .returning(|_, _| Ok(DiscordMessageId(12345)));

        let gateway = Arc::new(mock_gateway);

        // テスト用コンテナ構築（静的ディスパッチ）
        let container = AppContainer::new(gateway, db);

        // サービス実行
        let result = container.services.notification
            .send_notification(
                DiscordChannelId(111),
                "テストメッセージ",
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_recruitment_service() {
        let db = setup_test_db().await;

        let mut mock_gateway = MockDiscordGateway::new();

        // 複数のメソッドに期待値を設定
        mock_gateway
            .expect_send_message()
            .returning(|_, _| Ok(DiscordMessageId(12345)));

        mock_gateway
            .expect_add_reaction()
            .returning(|_, _, _| Ok(()));

        let gateway = Arc::new(mock_gateway);
        let container = AppContainer::new(gateway, db);

        let result = container.services.recruitment
            .create_recruitment(...)
            .await;

        assert!(result.is_ok());
    }
}
```

## 型エイリアスの活用

コードの可読性を高めるため、本番用の型エイリアスを定義する。

```rust
// src/di/mod.rs

pub use container::{AppContainer, ProductionAppContainer};
pub use services::{Services, ProductionServices};
pub use facades::{Facades, ProductionFacades};
pub use repositories::Repositories;

// 本番用エイリアス（Dataで使用）
pub type ProdContainer = ProductionAppContainer;

// src/types/data.rs
pub struct Data {
    pub container: ProdContainer,
}
```

## 完了条件

- [ ] Services<G>コンテナが実装されている
- [ ] Facades<G>コンテナが実装されている
- [ ] AppContainer<G>が実装されている
- [ ] 本番用型エイリアスが定義されている
- [ ] main.rsでDIコンテナが使用されている
- [ ] Arc<Http>がService/Facadeから除去されている
- [ ] Contextの直接使用がService/Facadeから除去されている
- [ ] テストでモックコンテナが使用可能

## 注意事項

1. **段階的移行** - 一度にすべてを変更せず、1サービスずつ移行
2. **後方互換性** - 移行中は古いパターンと新しいパターンが共存可能に
3. **循環参照に注意** - Arc/Weak適切に使用
4. **テスト容易性を優先** - モック注入が容易な設計に
5. **型パラメータの伝播** - 上位構造体にも型パラメータが必要
6. **型エイリアスを活用** - 可読性と保守性の向上

## 静的ディスパッチの利点（再掲）

1. **パフォーマンス** - vtable経由のオーバーヘッドなし、インライン化可能
2. **コンパイル時型チェック** - 誤った型の注入を防止
3. **最適化** - コンパイラによる積極的な最適化が可能
