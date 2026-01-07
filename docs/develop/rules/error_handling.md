# エラーハンドリングルール

> **📝 注意: 実装はよりシンプルです**
>
> このドキュメントに記載されているエラー変換パターンは理想的な設計を示していますが、
> 実際の実装はより単純化されています。
>
> **実装の詳細:**
> - エラー型: `src/errors/` ディレクトリ
> - `ServiceError`, `FacadeError`, `RepositoryError`, `PresentationError` に分類
> - エラー変換は `From` トレイトと `#[from]` 属性で実装
> - SeaORM の `DbErr` は直接 `ServiceError::Database(String)` に変換

## 基本方針

- **構造化エラー**: 各層で適切なエラー型を定義
- **エラー変換**: `#[from]`属性を使用した自動変換
- **ログ統合**: 構造化ログとエラートレースの連携
- **型安全性**: `thiserror`を使用した型安全で明確なエラーハンドリング

## エラーハンドリング戦略

### 層別エラーハンドリング

#### プレゼンテーション層

- **責務**: ユーザー向けエラーメッセージの生成
- **処理**: Facade層からのエラーをキャッチし、適切なDiscordメッセージに変換
- **ログ**: ユーザーアクションの失敗をログ出力

#### Facade層

- **責務**: ビジネス例外の捕捉とログ出力
- **処理**: Service層からのエラーをビジネス文脈に合わせて変換
- **トランザクション**: エラー時の自動ロールバック

#### Service層

- **責務**: ドメイン固有例外の生成
- **処理**: ビジネスルール違反時の明確なエラー生成
- **検証**: 入力値の検証とエラー生成

#### Repository層

- **責務**: データアクセス例外の適切な変換
- **処理**: データベースエラーをドメインエラーに変換
- **接続**: 接続エラー、制約違反エラーの処理

## エラー種別と対応方針

### 基本エラー分類

```rust
// エラーの階層化
pub enum ApplicationError {
    ValidationError(String),
    BusinessRuleViolation(String),
    DataAccessError(String),
    ExternalServiceError(String),
}
```

### 詳細なエラー型定義

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("必須フィールドが未入力です: {field}")]
    RequiredFieldMissing { field: String },

    #[error("フィールドの値が範囲外です: {field}, 値: {value}")]
    ValueOutOfRange { field: String, value: String },

    #[error("フィールドの形式が正しくありません: {field}")]
    InvalidFormat { field: String },
}

#[derive(Error, Debug)]
pub enum BusinessRuleError {
    #[error("募集が既に満員です: 募集ID {recruitment_id}")]
    RecruitmentFull { recruitment_id: String },

    #[error("権限がありません: 必要な権限 {required_permission}")]
    InsufficientPermission { required_permission: String },

    #[error("重複した操作です: {operation}")]
    DuplicateOperation { operation: String },
}

#[derive(Error, Debug)]
pub enum DataAccessError {
    #[error("データが見つかりません: {entity_type} ID {id}")]
    NotFound { entity_type: String, id: String },

    #[error("データベース接続エラー: {source}")]
    ConnectionError { #[from] source: sea_orm::DbErr },

    #[error("制約違反エラー: {constraint}")]
    ConstraintViolation { constraint: String },
}

#[derive(Error, Debug)]
pub enum ExternalServiceError {
    #[error("Discord APIエラー: {message}")]
    DiscordApiError { message: String },

    #[error("外部サービスタイムアウト: {service_name}")]
    ServiceTimeout { service_name: String },
}
```

## 層間エラー変換パターン

### Repository層からService層

```rust
impl From<sea_orm::DbErr> for DataAccessError {
    fn from(err: sea_orm::DbErr) -> Self {
        match err {
            sea_orm::DbErr::RecordNotFound(_) => DataAccessError::NotFound {
                entity_type: "Unknown".to_string(),
                id: "Unknown".to_string(),
            },
            _ => DataAccessError::ConnectionError { source: err },
        }
    }
}

impl From<DataAccessError> for BusinessRuleError {
    fn from(err: DataAccessError) -> Self {
        match err {
            DataAccessError::NotFound { entity_type, id } => {
                BusinessRuleError::DuplicateOperation {
                    operation: format!("{} {} の操作", entity_type, id),
                }
            }
            _ => BusinessRuleError::DuplicateOperation {
                operation: "データアクセスエラー".to_string(),
            },
        }
    }
}
```

### Service層からFacade層

```rust
#[derive(Error, Debug)]
pub enum FacadeError {
    #[error("ビジネスルールエラー: {source}")]
    BusinessRule { #[from] source: BusinessRuleError },

    #[error("バリデーションエラー: {source}")]
    Validation { #[from] source: ValidationError },

    #[error("外部サービスエラー: {source}")]
    ExternalService { #[from] source: ExternalServiceError },
}
```

### Facade層からプレゼンテーション層

```rust
impl From<FacadeError> for PoiseError {
    fn from(err: FacadeError) -> Self {
        match err {
            FacadeError::BusinessRule { source } => {
                PoiseError::from(format!("ビジネスエラー: {}", source))
            }
            FacadeError::Validation { source } => {
                PoiseError::from(format!("入力エラー: {}", source))
            }
            FacadeError::ExternalService { source } => {
                PoiseError::from(format!("外部サービスエラー: {}", source))
            }
        }
    }
}
```

## エラーログ戦略

### 構造化ログの実装

```rust
use tracing::{error, warn, info, debug};
use serde_json::json;

// ✅ 推奨: 構造化ログ
pub fn log_business_error(error: &BusinessRuleError, context: &str) {
    error!(
        target: "business_error",
        error = %error,
        context = context,
        "ビジネスルール違反が発生しました"
    );
}

pub fn log_validation_error(error: &ValidationError, user_id: &str) {
    warn!(
        target: "validation_error", 
        error = %error,
        user_id = user_id,
        "バリデーションエラーが発生しました"
    );
}

// ❌ 避けるべき: 非構造化ログ
pub fn bad_log_example(error: &str) {
    println!("エラーが発生しました: {}", error); // 避けるべき
}
```

### トランザクション内でのエラーハンドリング

```rust
impl TransactionManager {
    pub async fn execute_with_error_handling<T, F, Fut>(
        &self,
        operation: F,
    ) -> Result<T, FacadeError>
    where
        F: FnOnce(Transaction) -> Fut,
        Fut: Future<Output=Result<T, FacadeError>>,
    {
        let tx = self.begin_transaction().await?;

        match operation(tx).await {
            Ok(result) => {
                self.commit_transaction().await?;
                info!("トランザクションが正常に完了しました");
                Ok(result)
            }
            Err(err) => {
                self.rollback_transaction().await?;
                error!(
                    error = %err,
                    "トランザクション実行中にエラーが発生し、ロールバックしました"
                );
                Err(err)
            }
        }
    }
}
```

## エラーレスポンスパターン

### Discord用エラーレスポンス

```rust
pub async fn handle_command_error(
    ctx: &PoiseContext,
    error: FacadeError,
) -> Result<(), serenity::Error> {
    match error {
        FacadeError::Validation { source } => {
            ctx.say(format!("❌ 入力エラー: {}", source)).await?;
        }
        FacadeError::BusinessRule { source } => {
            ctx.say(format!("⚠️ 操作できません: {}", source)).await?;
        }
        FacadeError::ExternalService { source } => {
            ctx.say("🔧 一時的なエラーが発生しました。しばらく待ってから再試行してください。").await?;
            error!(error = %source, "外部サービスエラー");
        }
    }
    Ok(())
}
```

## 実装時の注意点

### Do's (推奨事項)

1. **具体的なエラーメッセージ**: ユーザーが理解できる具体的なメッセージを提供
2. **エラーの分類**: エラーの原因や種類を適切に分類
3. **ログの構造化**: 検索・分析しやすい構造化ログを使用
4. **エラー変換の明示化**: `#[from]`属性で自動変換を明示
5. **コンテキスト情報の保持**: エラー発生時の状況情報を保持

### Don'ts (避けるべき事項)

1. **パニックの使用**: `panic!`は避け、`Result`型を使用
2. **エラーの無視**: `unwrap()`や`expect()`の多用は避ける
3. **汎用的すぎるエラー**: `String`や`&str`だけのエラーは避ける
4. **機密情報の漏洩**: エラーメッセージに機密情報を含めない
5. **ログの重複**: 同一エラーの多重ログ出力は避ける

### エラーハンドリングのテスト戦略

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error_conversion() {
        let validation_err = ValidationError::RequiredFieldMissing {
            field: "quest_name".to_string(),
        };
        let facade_err = FacadeError::Validation { source: validation_err };
        let poise_err = PoiseError::from(facade_err);

        assert!(poise_err.to_string().contains("入力エラー"));
        assert!(poise_err.to_string().contains("quest_name"));
    }

    #[tokio::test]
    async fn test_transaction_rollback_on_error() {
        let tx_manager = setup_transaction_manager().await;

        let result = tx_manager.execute_with_error_handling(|_tx| async {
            Err(FacadeError::BusinessRule {
                source: BusinessRuleError::RecruitmentFull {
                    recruitment_id: "test123".to_string(),
                }
            })
        }).await;

        assert!(result.is_err());
        // ロールバックされていることを確認
        assert!(!tx_manager.is_transaction_active().await);
    }
}
```