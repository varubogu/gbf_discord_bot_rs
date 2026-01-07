# Discord API制約とクロージャパターン - アーキテクチャガイドライン

> **⚠️ 注意: これは設計提案ドキュメントです**
>
> このドキュメントに記載されているクロージャパターンやDiscord API制約は、理想的なアーキテクチャ設計を示すものですが、
> **現時点では実装されていません**。
>
> 実際の実装では、Facade層が `PoiseContext` を受け取り、Discord API操作を直接実行しています。
> このドキュメントは将来の設計改善の参考資料として保持されています。

## 概要

このドキュメントは、グランブルーファンタジーDiscord Botプロジェクトにおける**Discord API制約**と**クロージャパターン**
の詳細なガイドラインを定義します。

クリーンアーキテクチャの原則に従い、アプリケーション層以降でのDiscord
API処理を禁止し、プレゼンテーション層でのみDiscord操作を許可することで、純粋関数としてのFacade/Service層を実現し、テスタビリティと保守性を向上させます。

## Discord API制約の基本原則

### 制約の定義

```
アプリケーション層以降（Facade層、Service層、Repository層）では、
Discord API（poise/serenity）の直接呼び出しを禁止する
```

### 層別の責務と制約

#### プレゼンテーション層（Events/Commands/Handlers）

**許可事項**:

- PoiseContext、Serenity Contextの使用
- Discord API（メッセージ送信、編集、リアクション等）の直接呼び出し
- クロージャを通じたDiscord操作の実装

**禁止事項**:

- ビジネスロジックの実装
- データベース操作の直接実行

#### アプリケーション層（Facade/Service）

**許可事項**:

- 純粋なビジネスロジックの実装
- クロージャパラメータによるDiscord操作の要求
- トランザクション管理（Facade層のみ）

**禁止事項**:

- Discord API（poise/serenity）の直接呼び出し
- PoiseContext、Serenity Contextの受け取り
- 副作用を持つ外部システム操作

#### データアクセス層（Repository）

**許可事項**:

- データベース操作の実装
- エンティティ変換

**禁止事項**:

- Discord API操作
- ビジネスロジックの実装

## クロージャパターンの実装指針

### Discord操作の抽象化

```rust
/// Discord操作を抽象化する列挙型
pub enum DiscordOperation {
    SendMessage {
        channel_id: u64,
        content: String,
        embed: Option<CreateEmbed>,
    },
    EditMessage {
        channel_id: u64,
        message_id: u64,
        content: Option<String>,
        embed: Option<CreateEmbed>,
    },
    AddReaction {
        channel_id: u64,
        message_id: u64,
        emoji: String,
    },
    DeleteMessage {
        channel_id: u64,
        message_id: u64,
    },
    SendPrivateMessage {
        user_id: u64,
        content: String,
    },
}
```

### Facade層でのクロージャパターン実装

```rust
impl BattleRecruitmentFacade {
    /// 新しい募集を作成（クロージャパターン）
    pub async fn new_recruitment<F, Fut>(
        &self,
        quest_alias: &str,
        battle_type: BattleType,
        event_date: Option<DateTime<Utc>>,
        discord_operation: F,
    ) -> Result<RecruitmentResult>
    where
        F: Fn(DiscordOperation) -> Fut,
        Fut: Future<Output=Result<DiscordOperationResult>>,
    {
        // 1. クエスト情報の取得
        let quest_info = self.quest_service
            .get_quest_info(quest_alias)
            .await?;

        // 2. 募集データの作成
        let recruitment_data = self.new_service
            .create_recruitment_data(&quest_info, battle_type, event_date)
            .await?;

        // 3. Discord操作の要求（副作用を外部に委譲）
        let discord_result = discord_operation(DiscordOperation::SendMessage {
            channel_id: recruitment_data.channel_id,
            content: recruitment_data.message_content,
            embed: Some(recruitment_data.embed),
        }).await?;

        // 4. 結果をデータベースに保存
        let recruitment = self.new_service
            .save_recruitment(&recruitment_data, discord_result.message_id)
            .await?;

        Ok(RecruitmentResult {
            recruitment_id: recruitment.id,
            message_id: discord_result.message_id,
        })
    }
}
```

### プレゼンテーション層でのクロージャ実装

```rust
pub async fn recruit(ctx: PoiseContext<'_>, quest: String) -> Result<()> {
    ctx.defer().await?;

    let app_state = &ctx.data().app_state;
    let facade = BattleRecruitmentFacade::new(app_state);

    // クロージャを使用してDiscord操作を分離
    let result = facade.new_recruitment(
        &quest,
        BattleType::Default,
        None,
        |operation| {
            let ctx_clone = ctx.serenity_context().clone();
            Box::pin(async move {
                match operation {
                    DiscordOperation::SendMessage { channel_id, content, embed } => {
                        let channel = ChannelId::from(channel_id);
                        let message = channel
                            .send_message(&ctx_clone.http, |m| {
                                m.content(content);
                                if let Some(embed) = embed {
                                    m.set_embed(embed);
                                }
                                m
                            })
                            .await?;

                        Ok(DiscordOperationResult {
                            message_id: message.id.get(),
                        })
                    }
                    DiscordOperation::EditMessage { channel_id, message_id, content, embed } => {
                        let channel = ChannelId::from(channel_id);
                        channel.edit_message(&ctx_clone.http, message_id, |m| {
                            if let Some(content) = content {
                                m.content(content);
                            }
                            if let Some(embed) = embed {
                                m.set_embed(embed);
                            }
                            m
                        }).await?;

                        Ok(DiscordOperationResult { message_id })
                    }
                    // 他の操作の実装...
                }
            })
        }
    ).await?;

    ctx.say(format!("募集を作成しました。ID: {}", result.recruitment_id))
        .await?;
    Ok(())
}
```

## パフォーマンス最適化

### ゼロコスト抽象化の活用

```rust
// コンパイル時に最適化される関数型パターン
pub async fn new_recruitment<F, Fut>(
    &self,
    quest_alias: &str,
    discord_op: F,
) -> Result<RecruitmentResult>
where
    F: FnOnce(DiscordOperation) -> Fut + Send + 'static,
    Fut: Future<Output=Result<DiscordOperationResult>> + Send,
{
    // 実装...
}
```

### メモリ効率の考慮

```rust
// 不要なCloneを避ける
pub struct DiscordOperationContext<'a> {
    pub http: &'a Http,
    pub cache: &'a Cache,
}

// 参照を活用した効率的な実装
impl<'a> DiscordOperationContext<'a> {
    pub async fn execute_operation(
        &self,
        operation: &DiscordOperation,
    ) -> Result<DiscordOperationResult> {
        // 実装...
    }
}
```

## エラーハンドリング戦略

### Discord操作エラーの抽象化

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DiscordOperationError {
    #[error("メッセージの送信に失敗しました: {0}")]
    MessageSendFailed(String),

    #[error("メッセージの編集に失敗しました: {0}")]
    MessageEditFailed(String),

    #[error("権限が不足しています")]
    PermissionDenied,

    #[error("チャンネルが見つかりません")]
    ChannelNotFound,

    #[error("Discord API接続エラー: {0}")]
    ConnectionError(String),
}

impl From<SerenityError> for DiscordOperationError {
    fn from(err: SerenityError) -> Self {
        match err {
            SerenityError::Http(HttpError::UnsuccessfulRequest(ErrorResponse { status, .. })) => {
                match status {
                    403 => DiscordOperationError::PermissionDenied,
                    404 => DiscordOperationError::ChannelNotFound,
                    _ => DiscordOperationError::ConnectionError(err.to_string()),
                }
            }
            _ => DiscordOperationError::ConnectionError(err.to_string()),
        }
    }
}
```

### 層間エラー変換

```rust
// アプリケーション層のエラー
#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("バリデーションエラー: {0}")]
    ValidationError(String),

    #[error("ビジネスルール違反: {0}")]
    BusinessRuleViolation(String),

    #[error("データアクセスエラー: {0}")]
    DataAccessError(#[from] DataAccessError),

    #[error("外部サービスエラー: {0}")]
    ExternalServiceError(#[from] DiscordOperationError),
}
```

## テスト戦略

### Facade層のテスト

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn test_new_recruitment_success() {
        let facade = BattleRecruitmentFacade::new(&mock_app_state());
        let captured_operations = Arc::new(Mutex::new(Vec::new()));
        let captured_clone = captured_operations.clone();

        let result = facade.new_recruitment(
            "test_quest",
            BattleType::Default,
            None,
            |operation| {
                captured_clone.lock().unwrap().push(operation);
                Box::pin(async {
                    Ok(DiscordOperationResult { message_id: 12345 })
                })
            }
        ).await;

        assert!(result.is_ok());
        let operations = captured_operations.lock().unwrap();
        assert_eq!(operations.len(), 1);

        match &operations[0] {
            DiscordOperation::SendMessage { content, .. } => {
                assert!(content.contains("test_quest"));
            }
            _ => panic!("Expected SendMessage operation"),
        }
    }

    #[tokio::test]
    async fn test_new_recruitment_discord_error() {
        let facade = BattleRecruitmentFacade::new(&mock_app_state());

        let result = facade.new_recruitment(
            "test_quest",
            BattleType::Default,
            None,
            |_operation| {
                Box::pin(async {
                    Err(ApplicationError::ExternalServiceError(
                        DiscordOperationError::MessageSendFailed("Test error".to_string())
                    ))
                })
            }
        ).await;

        assert!(result.is_err());
    }
}
```

### Service層の純粋関数テスト

```rust
#[cfg(test)]
mod service_tests {
    use super::*;

    #[tokio::test]
    async fn test_create_recruitment_data_pure_function() {
        let service = NewRecruitmentService::new(mock_repository());
        let quest = Quest { name: "テストクエスト".to_string(), /* ... */ };

        let result = service.create_recruitment_data(
            &quest,
            BattleType::Default,
            None
        ).await;

        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.quest_name, "テストクエスト");
        // Discord操作に依存しない純粋なテストが可能
    }
}
```

## 実装時の注意事項

### 正しい実装パターン

```rust
// ✅ 正しいパターン（Facade層）
impl BattleRecruitmentFacade {
    pub async fn update_recruitment<F, Fut>(
        &self,
        recruitment_id: i32,
        new_content: String,
        discord_operation: F,
    ) -> Result<()>
    where
        F: FnOnce(DiscordOperation) -> Fut,
        Fut: Future<Output=Result<DiscordOperationResult>>,
    {
        // 純粋なビジネスロジック
        let recruitment = self.get_recruitment_service
            .get_by_id(recruitment_id)
            .await?;

        // Discord操作を外部に委譲
        discord_operation(DiscordOperation::EditMessage {
            channel_id: recruitment.channel_id as u64,
            message_id: recruitment.message_id as u64,
            content: Some(new_content),
            embed: None,
        }).await?;

        Ok(())
    }
}

// ✅ 正しいパターン（プレゼンテーション層）
pub async fn update_recruit(ctx: PoiseContext<'_>, message: Message) -> Result<()> {
    let facade = BattleRecruitmentFacade::new(&ctx.data().app_state);

    facade.update_recruitment(
        message.id.get() as i32,
        "更新されたメッセージ".to_string(),
        |operation| {
            let ctx = ctx.serenity_context().clone();
            Box::pin(async move {
                // Discord API操作の実装
                execute_discord_operation(&ctx, operation).await
            })
        }
    ).await
}
```

### 避けるべきアンチパターン

```rust
// ❌ アンチパターン（Facade層でのDiscord API使用）
impl BattleRecruitmentFacade {
    pub async fn new_recruitment_bad(
        &self,
        ctx: &PoiseContext<'_>,  // ❌ PoiseContextを受け取る
        quest: &str,
    ) -> Result<()> {
        // ビジネスロジック...

        // ❌ Facade層でのDiscord API直接呼び出し
        ctx.say("募集を作成しました").await?;

        Ok(())
    }
}

// ❌ アンチパターン（Service層でのDiscord API使用）
impl UpdateRecruitmentService {
    pub async fn update_recruitment_message_bad(
        &self,
        ctx: &Context,  // ❌ Serenity Contextを受け取る
        message_id: u64,
        content: String,
    ) -> Result<()> {
        // ❌ Service層でのDiscord API直接呼び出し
        let channel = ChannelId::from(12345);
        channel.edit_message(&ctx.http, message_id, |m| {
            m.content(content)
        }).await?;

        Ok(())
    }
}
```

## コードレビューの指針

### 遵守すべき制約の確認

- Facade/Service層でDiscord APIを直接呼び出していないこと
- PoiseContext/Serenity Contextがアプリケーション層に渡されていないこと
- クロージャパターンが適切に実装されていること
- エラーハンドリングが各層で適切に行われていること
- テストが純粋関数として実装されていること

## まとめ

このガイドラインに従うことで、以下の利益を得られます：

1. **テスタビリティの向上**: 純粋関数としてのFacade/Service層により、モックなしでのテストが可能
2. **保守性の向上**: 層間の責務が明確に分離され、変更の影響範囲が限定される
3. **パフォーマンスの向上**: Rustの零コスト抽象化により、実行時オーバーヘッドなし
4. **将来の拡張性**: Discord以外のプラットフォーム対応時にもアーキテクチャの変更が最小限

クリーンアーキテクチャとRustエコシステムの特性を最大限に活かした、持続可能で高性能なシステムを構築できます。