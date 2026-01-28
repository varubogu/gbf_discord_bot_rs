# Step 1: Discord Gateway抽象化層の導入

## 目的

poise/serenityへの直接依存をビジネスロジック層から排除するため、Gateway（またはAdapter）パターンを導入し、Discord APIとの通信を抽象化する。

## 概要

```
┌─────────────────────────────────────────────────────────────┐
│                      Events Layer                           │
│  (poise commands, event handlers - poise依存OK)             │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Gateway Layer (NEW)                       │
│  DiscordGateway trait + PoiseDiscordGateway impl            │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│              Facade / Service / Repository                   │
│         (Discord依存なし - Gateway traitのみ使用)            │
└─────────────────────────────────────────────────────────────┘
```

## 静的ディスパッチ vs 動的ディスパッチ

本計画では**静的ディスパッチ（ジェネリクス）**を採用する。

| 方式 | メリット | デメリット |
|------|---------|----------|
| 動的（`Arc<dyn Trait>`） | 型が統一される、コード簡潔 | vtable経由でオーバーヘッド |
| **静的（ジェネリクス）** | **インライン化可能、高速** | 型パラメータが増える |

## 作成するトレイト

### 1. DiscordMessageGateway

メッセージの送信・編集・削除を抽象化する。

```rust
// src/gateway/discord_message_gateway.rs

use async_trait::async_trait;
use crate::domain::types::{DiscordChannelId, DiscordMessageId, DiscordGuildId};
use crate::domain::models::message::{MessageContent, MessageData};
use crate::errors::GatewayError;

/// Discordメッセージ操作を抽象化するトレイト
#[async_trait]
pub trait DiscordMessageGateway: Send + Sync {
    /// メッセージを送信し、送信されたメッセージのIDを返す
    async fn send_message(
        &self,
        channel_id: DiscordChannelId,
        content: MessageContent,
    ) -> Result<DiscordMessageId, GatewayError>;

    /// メッセージを編集する
    async fn edit_message(
        &self,
        channel_id: DiscordChannelId,
        message_id: DiscordMessageId,
        content: MessageContent,
    ) -> Result<(), GatewayError>;

    /// メッセージを削除する
    async fn delete_message(
        &self,
        channel_id: DiscordChannelId,
        message_id: DiscordMessageId,
    ) -> Result<(), GatewayError>;

    /// メッセージを取得する
    async fn get_message(
        &self,
        channel_id: DiscordChannelId,
        message_id: DiscordMessageId,
    ) -> Result<MessageData, GatewayError>;

    /// チャンネル内のメッセージ一覧を取得する
    async fn get_messages(
        &self,
        channel_id: DiscordChannelId,
        limit: u8,
    ) -> Result<Vec<MessageData>, GatewayError>;
}
```

### 2. DiscordChannelGateway

チャンネルの作成・編集・削除を抽象化する。

```rust
// src/gateway/discord_channel_gateway.rs

use async_trait::async_trait;
use crate::domain::types::{DiscordChannelId, DiscordGuildId};
use crate::domain::models::channel::{ChannelCreateParams, ChannelEditParams, ChannelData};
use crate::errors::GatewayError;

/// Discordチャンネル操作を抽象化するトレイト
#[async_trait]
pub trait DiscordChannelGateway: Send + Sync {
    /// チャンネルを作成する
    async fn create_channel(
        &self,
        guild_id: DiscordGuildId,
        params: ChannelCreateParams,
    ) -> Result<DiscordChannelId, GatewayError>;

    /// チャンネルを編集する
    async fn edit_channel(
        &self,
        channel_id: DiscordChannelId,
        params: ChannelEditParams,
    ) -> Result<(), GatewayError>;

    /// チャンネルを削除する
    async fn delete_channel(
        &self,
        channel_id: DiscordChannelId,
    ) -> Result<(), GatewayError>;

    /// チャンネル情報を取得する
    async fn get_channel(
        &self,
        channel_id: DiscordChannelId,
    ) -> Result<ChannelData, GatewayError>;
}
```

### 3. DiscordInteractionGateway

インタラクション（ボタン、セレクトメニュー等）への応答を抽象化する。

```rust
// src/gateway/discord_interaction_gateway.rs

use async_trait::async_trait;
use crate::domain::types::DiscordInteractionId;
use crate::domain::models::interaction::{InteractionResponse, InteractionData};
use crate::errors::GatewayError;

/// Discordインタラクション操作を抽象化するトレイト
#[async_trait]
pub trait DiscordInteractionGateway: Send + Sync {
    /// インタラクションを遅延応答する
    async fn defer_interaction(
        &self,
        interaction_id: DiscordInteractionId,
    ) -> Result<(), GatewayError>;

    /// インタラクションに応答する
    async fn respond_to_interaction(
        &self,
        interaction_id: DiscordInteractionId,
        response: InteractionResponse,
    ) -> Result<(), GatewayError>;

    /// インタラクション応答を編集する
    async fn edit_interaction_response(
        &self,
        interaction_id: DiscordInteractionId,
        response: InteractionResponse,
    ) -> Result<(), GatewayError>;
}
```

### 4. DiscordReactionGateway

リアクションの追加・削除・取得を抽象化する。

```rust
// src/gateway/discord_reaction_gateway.rs

use async_trait::async_trait;
use crate::domain::types::{DiscordChannelId, DiscordMessageId, DiscordUserId};
use crate::domain::models::reaction::ReactionEmoji;
use crate::errors::GatewayError;

/// Discordリアクション操作を抽象化するトレイト
#[async_trait]
pub trait DiscordReactionGateway: Send + Sync {
    /// リアクションしたユーザー一覧を取得する
    async fn get_reaction_users(
        &self,
        channel_id: DiscordChannelId,
        message_id: DiscordMessageId,
        emoji: ReactionEmoji,
        limit: Option<u8>,
    ) -> Result<Vec<DiscordUserId>, GatewayError>;

    /// リアクションを追加する
    async fn add_reaction(
        &self,
        channel_id: DiscordChannelId,
        message_id: DiscordMessageId,
        emoji: ReactionEmoji,
    ) -> Result<(), GatewayError>;
}
```

### 5. DiscordGuildGateway

ギルド情報、メンバー、ロール取得を抽象化する。

```rust
// src/gateway/discord_guild_gateway.rs

use async_trait::async_trait;
use crate::domain::types::{DiscordGuildId, DiscordUserId, DiscordRoleId};
use crate::domain::models::guild::{GuildMember, GuildRole, GuildEmoji};
use crate::errors::GatewayError;

/// Discordギルド操作を抽象化するトレイト
#[async_trait]
pub trait DiscordGuildGateway: Send + Sync {
    /// ギルドメンバーを取得する
    async fn get_member(
        &self,
        guild_id: DiscordGuildId,
        user_id: DiscordUserId,
    ) -> Result<GuildMember, GatewayError>;

    /// ギルドロール一覧を取得する
    async fn get_roles(
        &self,
        guild_id: DiscordGuildId,
    ) -> Result<Vec<GuildRole>, GatewayError>;

    /// ギルド絵文字一覧を取得する
    async fn get_emojis(
        &self,
        guild_id: DiscordGuildId,
    ) -> Result<Vec<GuildEmoji>, GatewayError>;
}
```

## 統合Gatewayトレイト

複数のGatewayトレイトを1つにまとめたスーパートレイト。

```rust
// src/gateway/discord_gateway.rs

/// すべてのDiscord Gateway機能を統合したトレイト
pub trait DiscordGateway:
    DiscordMessageGateway
    + DiscordChannelGateway
    + DiscordInteractionGateway
    + DiscordReactionGateway
    + DiscordGuildGateway
{
}

// 自動実装
impl<T> DiscordGateway for T
where
    T: DiscordMessageGateway
        + DiscordChannelGateway
        + DiscordInteractionGateway
        + DiscordReactionGateway
        + DiscordGuildGateway,
{
}
```

## 実装クラス

### PoiseDiscordGateway

実際のpoise/serenityを使用する実装。本番環境で使用。

```rust
// src/gateway/impl/poise_discord_gateway.rs

use std::sync::Arc;
use async_trait::async_trait;
use poise::serenity_prelude::{Http, ChannelId, MessageId};
use crate::gateway::{
    DiscordMessageGateway, DiscordChannelGateway, DiscordInteractionGateway,
    DiscordReactionGateway, DiscordGuildGateway,
};
use crate::domain::types::{DiscordChannelId, DiscordMessageId};
use crate::domain::models::message::{MessageContent, MessageData};
use crate::errors::GatewayError;

/// Poise/Serenityを使用したDiscord Gateway実装
#[derive(Clone)]
pub struct PoiseDiscordGateway {
    http: Arc<Http>,
}

impl PoiseDiscordGateway {
    pub fn new(http: Arc<Http>) -> Self {
        Self { http }
    }
}

#[async_trait]
impl DiscordMessageGateway for PoiseDiscordGateway {
    async fn send_message(
        &self,
        channel_id: DiscordChannelId,
        content: MessageContent,
    ) -> Result<DiscordMessageId, GatewayError> {
        let serenity_channel_id = ChannelId::new(channel_id.0);

        // MessageContentからCreateMessageへの変換
        let create_message = content.into_serenity_message();

        let message = serenity_channel_id
            .send_message(&self.http, create_message)
            .await
            .map_err(|e| GatewayError::SendMessageFailed(e.to_string()))?;

        Ok(DiscordMessageId(message.id.get()))
    }

    // ... 他のメソッド実装
}

#[async_trait]
impl DiscordChannelGateway for PoiseDiscordGateway {
    // ... 実装
}

#[async_trait]
impl DiscordInteractionGateway for PoiseDiscordGateway {
    // ... 実装
}

#[async_trait]
impl DiscordReactionGateway for PoiseDiscordGateway {
    // ... 実装
}

#[async_trait]
impl DiscordGuildGateway for PoiseDiscordGateway {
    // ... 実装
}
```

### MockDiscordGateway

テスト用のモック実装。mockallを使用。

```rust
// src/gateway/impl/mock_discord_gateway.rs

use mockall::mock;
use async_trait::async_trait;
use crate::gateway::*;
use crate::domain::types::*;
use crate::domain::models::message::*;
use crate::errors::GatewayError;

mock! {
    /// テスト用モックGateway
    pub DiscordGateway {}

    #[async_trait]
    impl DiscordMessageGateway for DiscordGateway {
        async fn send_message(
            &self,
            channel_id: DiscordChannelId,
            content: MessageContent,
        ) -> Result<DiscordMessageId, GatewayError>;

        async fn edit_message(
            &self,
            channel_id: DiscordChannelId,
            message_id: DiscordMessageId,
            content: MessageContent,
        ) -> Result<(), GatewayError>;

        async fn delete_message(
            &self,
            channel_id: DiscordChannelId,
            message_id: DiscordMessageId,
        ) -> Result<(), GatewayError>;

        async fn get_message(
            &self,
            channel_id: DiscordChannelId,
            message_id: DiscordMessageId,
        ) -> Result<MessageData, GatewayError>;

        async fn get_messages(
            &self,
            channel_id: DiscordChannelId,
            limit: u8,
        ) -> Result<Vec<MessageData>, GatewayError>;
    }

    #[async_trait]
    impl DiscordChannelGateway for DiscordGateway {
        // ... 他のメソッド
    }

    #[async_trait]
    impl DiscordInteractionGateway for DiscordGateway {
        // ... 他のメソッド
    }

    #[async_trait]
    impl DiscordReactionGateway for DiscordGateway {
        // ... 他のメソッド
    }

    #[async_trait]
    impl DiscordGuildGateway for DiscordGateway {
        // ... 他のメソッド
    }
}
```

## ディレクトリ構成

```
src/
├── gateway/
│   ├── mod.rs                          # トレイト再エクスポート
│   ├── discord_gateway.rs              # 統合トレイト
│   ├── discord_message_gateway.rs      # メッセージ操作トレイト
│   ├── discord_channel_gateway.rs      # チャンネル操作トレイト
│   ├── discord_interaction_gateway.rs  # インタラクション操作トレイト
│   ├── discord_reaction_gateway.rs     # リアクション操作トレイト
│   ├── discord_guild_gateway.rs        # ギルド操作トレイト
│   └── impl/
│       ├── mod.rs
│       ├── poise_discord_gateway.rs    # 本番用実装
│       └── mock_discord_gateway.rs     # テスト用モック
```

## 静的ディスパッチでのService/Facade設計

### ジェネリクスを使用したService定義

```rust
// src/services/schedule/notification_service.rs

use std::sync::Arc;
use crate::gateway::DiscordMessageGateway;
use crate::domain::types::DiscordChannelId;
use crate::domain::models::message::MessageContent;
use crate::errors::ServiceError;

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
    ) -> Result<(), ServiceError> {
        let content = MessageContent::text(text);
        self.message_gateway
            .send_message(channel_id, content)
            .await
            .map_err(ServiceError::from)?;
        Ok(())
    }
}

// 型エイリアスで本番用の型を定義
pub type ProductionNotificationService = NotificationService<PoiseDiscordGateway>;
```

### 複数のGatewayを使用するService

```rust
// src/services/recruitment/participants.rs

use std::sync::Arc;
use crate::gateway::{DiscordMessageGateway, DiscordReactionGateway};
use crate::domain::types::{DiscordChannelId, DiscordMessageId, DiscordUserId};
use crate::errors::ServiceError;

/// 参加者管理サービス（複数Gateway使用）
pub struct ParticipantsService<MG, RG>
where
    MG: DiscordMessageGateway,
    RG: DiscordReactionGateway,
{
    message_gateway: Arc<MG>,
    reaction_gateway: Arc<RG>,
}

impl<MG, RG> ParticipantsService<MG, RG>
where
    MG: DiscordMessageGateway,
    RG: DiscordReactionGateway,
{
    pub fn new(message_gateway: Arc<MG>, reaction_gateway: Arc<RG>) -> Self {
        Self {
            message_gateway,
            reaction_gateway,
        }
    }

    pub async fn get_participants(
        &self,
        channel_id: DiscordChannelId,
        message_id: DiscordMessageId,
    ) -> Result<Vec<DiscordUserId>, ServiceError> {
        // リアクションからユーザー一覧を取得
        let users = self.reaction_gateway
            .get_reaction_users(
                channel_id,
                message_id,
                ReactionEmoji::unicode("✅"),
                None,
            )
            .await?;

        Ok(users)
    }
}

// 本番用型エイリアス（同一実装なので1つの型パラメータで済む）
pub type ProductionParticipantsService = ParticipantsService<PoiseDiscordGateway, PoiseDiscordGateway>;
```

### 統合Gatewayトレイトを使用したService

複数のGatewayメソッドを使用する場合、統合トレイトを使うとシンプルになる。

```rust
// src/services/recruitment/recruitment_service.rs

use std::sync::Arc;
use crate::gateway::DiscordGateway;  // 統合トレイト

/// 募集サービス（統合Gateway使用）
pub struct RecruitmentService<G>
where
    G: DiscordGateway,  // 全機能が使える
{
    gateway: Arc<G>,
    repository: Arc<BattleRecruitmentsRepository>,
}

impl<G> RecruitmentService<G>
where
    G: DiscordGateway,
{
    pub fn new(gateway: Arc<G>, repository: Arc<BattleRecruitmentsRepository>) -> Self {
        Self { gateway, repository }
    }

    pub async fn create_recruitment(&self, ...) -> Result<i64, ServiceError> {
        // メッセージ送信（DiscordMessageGateway）
        let message_id = self.gateway.send_message(...).await?;

        // リアクション追加（DiscordReactionGateway）
        self.gateway.add_reaction(...).await?;

        // DB保存
        let id = self.repository.create(...).await?;

        Ok(id)
    }
}

// 本番用型エイリアス
pub type ProductionRecruitmentService = RecruitmentService<PoiseDiscordGateway>;
```

## 移行パターン

### Before: Httpを直接使用

```rust
// 変更前: サービスがHttpを直接使用
pub struct NotificationService {
    http: Arc<Http>,
}

impl NotificationService {
    pub async fn send_notification(&self, channel_id: u64, text: &str) {
        let channel = ChannelId::new(channel_id);
        let message = CreateMessage::new().content(text);
        channel.send_message(&self.http, message).await.unwrap();
    }
}
```

### After: ジェネリクスでGatewayトレイトを使用

```rust
// 変更後: サービスはジェネリクスでGatewayトレイトに依存
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
    ) -> Result<(), ServiceError> {
        let content = MessageContent::text(text);
        self.message_gateway
            .send_message(channel_id, content)
            .await
            .map_err(ServiceError::from)?;
        Ok(())
    }
}

// 型エイリアスで具象型を定義
pub type ProductionNotificationService = NotificationService<PoiseDiscordGateway>;
```

## テストでの使用

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::gateway::impl::MockDiscordGateway;

    #[tokio::test]
    async fn test_notification_service() {
        // モックGatewayを作成
        let mut mock_gateway = MockDiscordGateway::new();

        // 期待値設定
        mock_gateway
            .expect_send_message()
            .returning(|_, _| Ok(DiscordMessageId(12345)));

        // サービスにモックを注入（静的ディスパッチ）
        let service = NotificationService::new(Arc::new(mock_gateway));

        // テスト実行
        let result = service
            .send_notification(DiscordChannelId(111), "テスト")
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_participants_service() {
        let mut mock_gateway = MockDiscordGateway::new();

        mock_gateway
            .expect_get_reaction_users()
            .returning(|_, _, _, _| Ok(vec![
                DiscordUserId(1001),
                DiscordUserId(1002),
            ]));

        // 同じモックを両方のジェネリクスに使用
        let gateway = Arc::new(mock_gateway);
        let service = ParticipantsService::new(gateway.clone(), gateway);

        let result = service
            .get_participants(DiscordChannelId(111), DiscordMessageId(222))
            .await;

        assert_eq!(result.unwrap().len(), 2);
    }
}
```

## DIコンテナ（静的ディスパッチ版）

```rust
// src/di/container.rs

use std::sync::Arc;
use crate::gateway::impl::PoiseDiscordGateway;

/// 本番環境用コンテナ（具象型を使用）
pub struct AppContainer {
    pub gateway: Arc<PoiseDiscordGateway>,
    pub notification_service: Arc<NotificationService<PoiseDiscordGateway>>,
    pub recruitment_service: Arc<RecruitmentService<PoiseDiscordGateway>>,
    pub participants_service: Arc<ParticipantsService<PoiseDiscordGateway, PoiseDiscordGateway>>,
}

impl AppContainer {
    pub fn new(http: Arc<Http>, db: Arc<DatabaseConnection>) -> Self {
        let gateway = Arc::new(PoiseDiscordGateway::new(http));
        let repositories = Repositories::new(db);

        let notification_service = Arc::new(
            NotificationService::new(gateway.clone())
        );

        let recruitment_service = Arc::new(
            RecruitmentService::new(
                gateway.clone(),
                repositories.battle_recruitments.clone(),
            )
        );

        let participants_service = Arc::new(
            ParticipantsService::new(gateway.clone(), gateway.clone())
        );

        Self {
            gateway,
            notification_service,
            recruitment_service,
            participants_service,
        }
    }
}
```

## 完了条件

- [ ] 5つのGatewayトレイトが定義されている
- [ ] 統合Gatewayトレイト（DiscordGateway）が定義されている
- [ ] PoiseDiscordGateway実装が完成している
- [ ] MockDiscordGateway（mockall）が定義されている
- [ ] Service/Facadeがジェネリクスで定義されている
- [ ] 本番用型エイリアスが定義されている

## 注意事項

1. **トレイトは最小限のメソッドで開始** - 必要に応じて追加する
2. **エラー型は統一** - `GatewayError`を定義して使用
3. **型変換はGateway実装内で行う** - ドメイン型⇔serenity型の変換
4. **非同期トレイトには`async_trait`クレートを使用**
5. **型パラメータが多くなる場合は統合トレイトを使用**
6. **本番用型エイリアスを定義して可読性を確保**

## 静的ディスパッチの利点

1. **パフォーマンス** - vtable経由のオーバーヘッドなし、インライン化可能
2. **コンパイル時型チェック** - 誤った型の注入を防止
3. **最適化** - コンパイラによる積極的な最適化が可能

## 静的ディスパッチの注意点

1. **型パラメータの伝播** - 上位の構造体にも型パラメータが必要
2. **コンパイル時間** - ジェネリクスによるモノモーフィゼーションで増加の可能性
3. **コード量** - where句などで記述量が増える

これらの注意点は、型エイリアスの活用と統合トレイトの使用で軽減できる。
