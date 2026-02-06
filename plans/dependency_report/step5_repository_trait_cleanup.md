# Step 5: Repositoryトレイト定義の修正

## 目的

Repositoryトレイト定義に含まれるpoise/serenity型（`MessageId`等）をプリミティブ型またはドメイン型に置き換え、Repository層の独立性を確保する。

## 概要

```
┌─────────────────────────────────────────────────────────────┐
│                    現在の問題                                │
│  Repository trait が poise::MessageId を使用                 │
│  → Repositoryがpoise/serenityに依存                          │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    修正後                                    │
│  Repository trait は u64 または DiscordMessageId を使用      │
│  → Repositoryはドメイン型のみに依存                          │
└─────────────────────────────────────────────────────────────┘
```

## 修正対象

### battle_recruitments_repository.rs

現在の問題箇所：

```rust
// 変更前
use poise::serenity_prelude::MessageId;

#[async_trait]
pub trait BattleRecruitmentsRepository: Send + Sync {
    async fn set_end_message(
        &self,
        id: i64,
        message_id: MessageId,  // ← poise型
    ) -> Result<(), RepositoryError>;

    async fn set_canceled_with_txn(
        &self,
        txn: &DatabaseTransaction,
        id: i64,
        message_id: MessageId,  // ← poise型
    ) -> Result<(), RepositoryError>;
}
```

## 修正パターン

### パターン1: プリミティブ型（u64）を使用

最もシンプルな修正。型安全性は低いが、外部依存を完全に排除。

```rust
// src/repository/battle_recruitments_repository.rs

#[async_trait]
pub trait BattleRecruitmentsRepository: Send + Sync {
    /// 終了メッセージIDを設定する
    async fn set_end_message(
        &self,
        id: i64,
        message_id: u64,  // プリミティブ型
    ) -> Result<(), RepositoryError>;

    /// キャンセル状態を設定する（トランザクション内）
    async fn set_canceled_with_txn(
        &self,
        txn: &DatabaseTransaction,
        id: i64,
        message_id: u64,  // プリミティブ型
    ) -> Result<(), RepositoryError>;

    // その他のメソッド...
}

// 実装側
impl BattleRecruitmentsRepository for BattleRecruitmentsRepositoryImpl {
    async fn set_end_message(
        &self,
        id: i64,
        message_id: u64,
    ) -> Result<(), RepositoryError> {
        // message_idをそのままDB保存
        // ...
    }
}

// 呼び出し側（Service）
impl RecruitmentService {
    pub async fn set_end_message(&self, id: i64, message_id: DiscordMessageId) {
        // ドメイン型からu64に変換
        self.repository.set_end_message(id, message_id.get()).await
    }
}
```

### パターン2: ドメイン型（DiscordMessageId）を使用

型安全性を維持しつつ外部依存を排除。推奨パターン。

```rust
// src/repository/battle_recruitments_repository.rs

use crate::domain::types::DiscordMessageId;

#[async_trait]
pub trait BattleRecruitmentsRepository: Send + Sync {
    /// 終了メッセージIDを設定する
    async fn set_end_message(
        &self,
        id: i64,
        message_id: DiscordMessageId,  // ドメイン型
    ) -> Result<(), RepositoryError>;

    /// キャンセル状態を設定する（トランザクション内）
    async fn set_canceled_with_txn(
        &self,
        txn: &DatabaseTransaction,
        id: i64,
        message_id: DiscordMessageId,  // ドメイン型
    ) -> Result<(), RepositoryError>;
}

// 実装側
impl BattleRecruitmentsRepository for BattleRecruitmentsRepositoryImpl {
    async fn set_end_message(
        &self,
        id: i64,
        message_id: DiscordMessageId,
    ) -> Result<(), RepositoryError> {
        // ドメイン型から内部値を取得してDB保存
        let msg_id_value = message_id.get() as i64;

        battle_recruitments::ActiveModel {
            id: Set(id),
            end_message_id: Set(Some(msg_id_value)),
            ..Default::default()
        }
        .update(&self.db)
        .await?;

        Ok(())
    }
}
```

## 全体的なRepository設計の見直し

### IDパラメータの統一

すべてのDiscord関連IDをドメイン型に統一する。

```rust
// src/repository/traits.rs

use crate::domain::types::{
    DiscordChannelId,
    DiscordMessageId,
    DiscordGuildId,
    DiscordUserId,
};

#[async_trait]
pub trait BattleRecruitmentsRepository: Send + Sync {
    /// IDで募集を取得
    async fn find_by_id(&self, id: i64) -> Result<Option<Model>, RepositoryError>;

    /// チャンネルIDで募集一覧を取得
    async fn find_by_channel(
        &self,
        channel_id: DiscordChannelId,  // ドメイン型
    ) -> Result<Vec<Model>, RepositoryError>;

    /// ギルドIDで募集一覧を取得
    async fn find_by_guild(
        &self,
        guild_id: DiscordGuildId,  // ドメイン型
    ) -> Result<Vec<Model>, RepositoryError>;

    /// メッセージIDで募集を取得
    async fn find_by_message(
        &self,
        message_id: DiscordMessageId,  // ドメイン型
    ) -> Result<Option<Model>, RepositoryError>;

    /// 終了メッセージIDを設定
    async fn set_end_message(
        &self,
        id: i64,
        message_id: DiscordMessageId,  // ドメイン型
    ) -> Result<(), RepositoryError>;

    /// キャンセル状態を設定（トランザクション内）
    async fn set_canceled_with_txn(
        &self,
        txn: &DatabaseTransaction,
        id: i64,
        message_id: DiscordMessageId,  // ドメイン型
    ) -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait GuildSettingsRepository: Send + Sync {
    /// ギルド設定を取得
    async fn find_by_guild(
        &self,
        guild_id: DiscordGuildId,  // ドメイン型
    ) -> Result<Option<Model>, RepositoryError>;

    /// ギルド設定を保存
    async fn save(
        &self,
        guild_id: DiscordGuildId,  // ドメイン型
        settings: GuildSettingsData,
    ) -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait ChannelsRepository: Send + Sync {
    /// チャンネル情報を取得
    async fn find_by_id(
        &self,
        channel_id: DiscordChannelId,  // ドメイン型
    ) -> Result<Option<Model>, RepositoryError>;

    /// ギルド内のチャンネル一覧を取得
    async fn find_by_guild(
        &self,
        guild_id: DiscordGuildId,  // ドメイン型
    ) -> Result<Vec<Model>, RepositoryError>;
}
```

## 入出力モデルの定義

Repositoryの入出力にはドメインモデルまたはエンティティを使用し、Discord型を含めない。

```rust
// src/repository/models.rs

use crate::domain::types::*;
use chrono::{DateTime, Utc};

/// 募集作成データ
#[derive(Debug, Clone)]
pub struct CreateRecruitmentData {
    pub guild_id: DiscordGuildId,
    pub channel_id: DiscordChannelId,
    pub message_id: DiscordMessageId,
    pub owner_id: DiscordUserId,
    pub quest_name: String,
    pub battle_style: String,
    pub max_participants: i32,
    pub start_time: DateTime<Utc>,
}

/// 募集更新データ
#[derive(Debug, Clone, Default)]
pub struct UpdateRecruitmentData {
    pub quest_name: Option<String>,
    pub battle_style: Option<String>,
    pub max_participants: Option<i32>,
    pub start_time: Option<DateTime<Utc>>,
    pub status: Option<String>,
    pub end_message_id: Option<DiscordMessageId>,
}

// Repositoryトレイトでの使用
#[async_trait]
pub trait BattleRecruitmentsRepository: Send + Sync {
    /// 募集を作成
    async fn create(
        &self,
        data: CreateRecruitmentData,
    ) -> Result<i64, RepositoryError>;

    /// 募集を更新
    async fn update(
        &self,
        id: i64,
        data: UpdateRecruitmentData,
    ) -> Result<(), RepositoryError>;
}
```

## Entity定義の見直し

SeaORMのEntity定義自体はDB構造を反映するため変更不要だが、変換メソッドを追加。

```rust
// src/entity/battle_recruitments.rs

use crate::domain::types::*;

impl Model {
    /// ギルドIDをドメイン型で取得
    pub fn guild_id(&self) -> DiscordGuildId {
        DiscordGuildId(self.guild_id as u64)
    }

    /// チャンネルIDをドメイン型で取得
    pub fn channel_id(&self) -> DiscordChannelId {
        DiscordChannelId(self.channel_id as u64)
    }

    /// メッセージIDをドメイン型で取得
    pub fn message_id(&self) -> DiscordMessageId {
        DiscordMessageId(self.message_id as u64)
    }

    /// オーナーIDをドメイン型で取得
    pub fn owner_id(&self) -> DiscordUserId {
        DiscordUserId(self.owner_id as u64)
    }

    /// 終了メッセージIDをドメイン型で取得
    pub fn end_message_id(&self) -> Option<DiscordMessageId> {
        self.end_message_id.map(|id| DiscordMessageId(id as u64))
    }
}
```

## 実装例

### 修正後のRepository実装

```rust
// src/repository/impl/battle_recruitments_repository_impl.rs

use crate::domain::types::*;
use crate::repository::{BattleRecruitmentsRepository, CreateRecruitmentData, UpdateRecruitmentData};
use crate::entity::battle_recruitments;

pub struct BattleRecruitmentsRepositoryImpl {
    db: Arc<DatabaseConnection>,
}

#[async_trait]
impl BattleRecruitmentsRepository for BattleRecruitmentsRepositoryImpl {
    async fn find_by_channel(
        &self,
        channel_id: DiscordChannelId,
    ) -> Result<Vec<battle_recruitments::Model>, RepositoryError> {
        let results = battle_recruitments::Entity::find()
            .filter(battle_recruitments::Column::ChannelId.eq(channel_id.get() as i64))
            .all(&*self.db)
            .await?;

        Ok(results)
    }

    async fn create(
        &self,
        data: CreateRecruitmentData,
    ) -> Result<i64, RepositoryError> {
        let model = battle_recruitments::ActiveModel {
            guild_id: Set(data.guild_id.get() as i64),
            channel_id: Set(data.channel_id.get() as i64),
            message_id: Set(data.message_id.get() as i64),
            owner_id: Set(data.owner_id.get() as i64),
            quest_name: Set(data.quest_name),
            battle_style: Set(data.battle_style),
            max_participants: Set(data.max_participants),
            start_time: Set(data.start_time.naive_utc()),
            status: Set("open".to_string()),
            ..Default::default()
        };

        let result = model.insert(&*self.db).await?;
        Ok(result.id)
    }

    async fn set_end_message(
        &self,
        id: i64,
        message_id: DiscordMessageId,
    ) -> Result<(), RepositoryError> {
        battle_recruitments::Entity::update_many()
            .filter(battle_recruitments::Column::Id.eq(id))
            .col_expr(
                battle_recruitments::Column::EndMessageId,
                Expr::value(message_id.get() as i64),
            )
            .exec(&*self.db)
            .await?;

        Ok(())
    }

    async fn set_canceled_with_txn(
        &self,
        txn: &DatabaseTransaction,
        id: i64,
        message_id: DiscordMessageId,
    ) -> Result<(), RepositoryError> {
        battle_recruitments::Entity::update_many()
            .filter(battle_recruitments::Column::Id.eq(id))
            .col_expr(
                battle_recruitments::Column::Status,
                Expr::value("cancelled"),
            )
            .col_expr(
                battle_recruitments::Column::EndMessageId,
                Expr::value(message_id.get() as i64),
            )
            .exec(txn)
            .await?;

        Ok(())
    }
}
```

### 呼び出し側（Service）の修正

```rust
// src/services/recruitment/cancel.rs（変更後）

use crate::domain::types::DiscordMessageId;
use crate::repository::BattleRecruitmentsRepository;

impl RecruitmentCancelService {
    pub async fn cancel_recruitment(
        &self,
        recruitment_id: i64,
        end_message_id: DiscordMessageId,  // ドメイン型を受け取る
    ) -> Result<(), ServiceError> {
        let txn = self.db.begin().await?;

        // Repositoryにはドメイン型をそのまま渡す
        self.repository
            .set_canceled_with_txn(&txn, recruitment_id, end_message_id)
            .await?;

        txn.commit().await?;
        Ok(())
    }
}
```

## ディレクトリ構成

```
src/repository/
├── mod.rs
├── traits.rs                    # Repositoryトレイト定義
├── models.rs                    # 入出力モデル定義
├── error.rs                     # RepositoryError定義
└── impl/
    ├── mod.rs
    ├── battle_recruitments_repository_impl.rs
    ├── guild_settings_repository_impl.rs
    └── channels_repository_impl.rs
```

## 完了条件

- [ ] `poise::serenity_prelude::MessageId`がRepositoryトレイトから除去されている
- [ ] すべてのDiscord ID型がドメイン型（または`u64`）に置き換わっている
- [ ] Repository入出力モデルが定義されている
- [ ] Entity Modelに変換ヘルパーメソッドが追加されている
- [ ] 呼び出し側（Service）が修正されている
- [ ] テストが通過する

## 注意事項

1. **DBスキーマは変更不要** - カラム型（i64等）はそのまま
2. **Entity定義は変更最小限** - 変換ヘルパーの追加のみ
3. **トランザクション対応を維持** - `_with_txn`メソッドのシグネチャを保持
4. **既存のクエリロジックは維持** - フィルタ条件の書き方は同じ

## 移行チェックリスト

1. [ ] `DiscordMessageId`型が`src/domain/types/`に定義済み
2. [ ] `BattleRecruitmentsRepository`トレイトのシグネチャ修正
3. [ ] `BattleRecruitmentsRepositoryImpl`の実装修正
4. [ ] 呼び出し箇所（Service/Facade）の修正
5. [ ] 既存テストの修正
6. [ ] 新しい型に対するテスト追加
