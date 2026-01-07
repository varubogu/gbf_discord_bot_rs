# エラー型定義設計書

> **⚠️ 注意: これは設計提案ドキュメントです**
>
> このドキュメントに記載されている詳細なエラー型階層は、理想的な設計を示すものですが、**実際にはより簡潔な実装**になっています。
>
> **実際の実装:**
> - `AppError` (`src/types/error.rs`): アプリケーション全体のメインエラー型
> - `ServiceError`, `FacadeError`, `RepositoryError`, `PresentationError`: 層別のエラー型
> - `thiserror` クレートを使用した基本的なエラー変換
>
> 実装の詳細は `src/types/error.rs` および `src/errors/` ディレクトリを参照してください。

## 概要

GBF Discord Bot全体で使用するエラー型の定義と階層構造を定義します。`thiserror`クレートを使用した型安全で明確なエラーハンドリングを実現します。

## エラー型の階層構造

```
ApplicationError (最上位)
├── PresentationError (プレゼンテーション層)
├── FacadeError (Facade層)
├── ServiceError (Service層)
│   ├── ValidationError
│   ├── BusinessRuleError
│   └── ExternalServiceError
└── RepositoryError (Repository層)
    └── DataAccessError
```

## 層別エラー型定義

### 1. Repository層エラー

データアクセス関連のエラー。

```rust
use thiserror::Error;
use sea_orm::DbErr;

#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("データが見つかりません: {entity_type} (ID: {id})")]
    NotFound {
        entity_type: String,
        id: String,
    },

    #[error("データベース接続エラー")]
    ConnectionError {
        #[from]
        source: DbErr,
    },

    #[error("トランザクションエラー: {message}")]
    TransactionError {
        message: String,
    },

    #[error("制約違反エラー: {constraint}")]
    ConstraintViolation {
        constraint: String,
    },

    #[error("データベースクエリエラー: {query}")]
    QueryError {
        query: String,
        #[source]
        source: DbErr,
    },
}
```

---

### 2. Service層エラー

ビジネスロジック関連のエラー。

#### ValidationError（バリデーションエラー）

```rust
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("必須フィールドが未入力です: {field}")]
    RequiredFieldMissing {
        field: String,
    },

    #[error("フィールドの値が範囲外です: {field} (値: {value}, 許容範囲: {range})")]
    ValueOutOfRange {
        field: String,
        value: String,
        range: String,
    },

    #[error("フィールドの形式が正しくありません: {field} (理由: {reason})")]
    InvalidFormat {
        field: String,
        reason: String,
    },

    #[error("データ型変換エラー: {field} (値: {value}, 期待される型: {expected_type})")]
    TypeConversionError {
        field: String,
        value: String,
        expected_type: String,
    },

    #[error("日時形式エラー: {value} (対応フォーマット: {supported_formats})")]
    DateTimeFormatError {
        value: String,
        supported_formats: String,
    },

    #[error("UUID形式エラー: {value}")]
    UuidFormatError {
        value: String,
    },

    #[error("外部キー制約エラー: {field} (参照先: {reference_table}, 値: {value})")]
    ForeignKeyViolation {
        field: String,
        reference_table: String,
        value: String,
    },
}
```

#### BusinessRuleError（ビジネスルール違反エラー）

```rust
#[derive(Error, Debug)]
pub enum BusinessRuleError {
    #[error("権限がありません: {required_permission}")]
    InsufficientPermission {
        required_permission: String,
    },

    #[error("募集が既に満員です (募集ID: {recruitment_id}, 定員: {capacity})")]
    RecruitmentFull {
        recruitment_id: String,
        capacity: i32,
    },

    #[error("重複した操作です: {operation}")]
    DuplicateOperation {
        operation: String,
    },

    #[error("操作対象が不正な状態です: {entity} (現在の状態: {current_state})")]
    InvalidState {
        entity: String,
        current_state: String,
    },

    #[error("ギルドIDが一致しません (期待: {expected}, 実際: {actual})")]
    GuildIdMismatch {
        expected: String,
        actual: String,
    },

    #[error("テーブル定義エラー: {table_name} (理由: {reason})")]
    TableDefinitionError {
        table_name: String,
        reason: String,
    },
}
```

#### ExternalServiceError（外部サービスエラー）

```rust
#[derive(Error, Debug)]
pub enum ExternalServiceError {
    #[error("Discord APIエラー: {message}")]
    DiscordApiError {
        message: String,
        #[source]
        source: Option<serenity::Error>,
    },

    #[error("Google Sheets APIエラー: {message}")]
    GoogleSheetsApiError {
        message: String,
    },

    #[error("Google認証エラー: {message}")]
    GoogleAuthError {
        message: String,
    },

    #[error("スプレッドシートが見つかりません: {spreadsheet_url}")]
    SpreadsheetNotFound {
        spreadsheet_url: String,
    },

    #[error("シートが見つかりません: {sheet_name} (スプレッドシート: {spreadsheet_id})")]
    SheetNotFound {
        sheet_name: String,
        spreadsheet_id: String,
    },

    #[error("外部サービスタイムアウト: {service_name} (タイムアウト: {timeout_seconds}秒)")]
    ServiceTimeout {
        service_name: String,
        timeout_seconds: u64,
    },

    #[error("ネットワークエラー: {message}")]
    NetworkError {
        message: String,
    },
}
```

---

### 3. Facade層エラー

複数のService層エラーを統合。

```rust
#[derive(Error, Debug)]
pub enum FacadeError {
    #[error("バリデーションエラー")]
    Validation {
        #[from]
        source: ValidationError,
    },

    #[error("ビジネスルールエラー")]
    BusinessRule {
        #[from]
        source: BusinessRuleError,
    },

    #[error("外部サービスエラー")]
    ExternalService {
        #[from]
        source: ExternalServiceError,
    },

    #[error("データアクセスエラー")]
    Repository {
        #[from]
        source: RepositoryError,
    },

    #[error("トランザクションエラー: {message}")]
    TransactionError {
        message: String,
    },
}
```

---

### 4. Presentation層エラー

Discordユーザー向けエラーメッセージ。

```rust
#[derive(Error, Debug)]
pub enum PresentationError {
    #[error("{message}")]
    UserFacingError {
        message: String,
        #[source]
        source: Option<FacadeError>,
    },
}

impl From<FacadeError> for PresentationError {
    fn from(err: FacadeError) -> Self {
        let message = match &err {
            FacadeError::Validation { source } => {
                format!("❌ 入力エラー: {}", source)
            }
            FacadeError::BusinessRule { source } => {
                format!("⚠️ 操作できません: {}", source)
            }
            FacadeError::ExternalService { source } => {
                match source {
                    ExternalServiceError::ServiceTimeout { .. } => {
                        "🔧 タイムアウトが発生しました。しばらく待ってから再試行してください。".to_string()
                    }
                    ExternalServiceError::GoogleSheetsApiError { .. } => {
                        "🔧 Googleスプレッドシートへのアクセスに失敗しました。".to_string()
                    }
                    _ => {
                        "🔧 外部サービスでエラーが発生しました。".to_string()
                    }
                }
            }
            FacadeError::Repository { .. } => {
                "🔧 データベースエラーが発生しました。管理者に連絡してください。".to_string()
            }
            FacadeError::TransactionError { .. } => {
                "🔧 処理に失敗しました。再試行してください。".to_string()
            }
        };

        PresentationError::UserFacingError {
            message,
            source: Some(err),
        }
    }
}
```

---

## エラー変換パターン

### Repository → Service

```rust
impl From<RepositoryError> for BusinessRuleError {
    fn from(err: RepositoryError) -> Self {
        match err {
            RepositoryError::NotFound { entity_type, id } => {
                BusinessRuleError::InvalidState {
                    entity: entity_type,
                    current_state: format!("ID {} が見つかりません", id),
                }
            }
            RepositoryError::ConstraintViolation { constraint } => {
                BusinessRuleError::DuplicateOperation {
                    operation: format!("制約違反: {}", constraint),
                }
            }
            _ => BusinessRuleError::InvalidState {
                entity: "Unknown".to_string(),
                current_state: "データアクセスエラー".to_string(),
            },
        }
    }
}
```

### Service → Facade

```rust
// ValidationError, BusinessRuleError, ExternalServiceError は
// #[from] 属性により自動変換される
```

### Facade → Presentation

```rust
// PresentationError の From<FacadeError> 実装により自動変換
```

---

## スプレッドシート機能固有のエラー

スプレッドシート機能では、既存のエラー型を活用します。

### データ変換エラー

`ValidationError` を使用：
- `TypeConversionError` - 型変換失敗
- `DateTimeFormatError` - 日時形式エラー
- `UuidFormatError` - UUID形式エラー
- `ForeignKeyViolation` - 外部キー制約違反

### スプレッドシートアクセスエラー

`ExternalServiceError` を使用：
- `GoogleSheetsApiError` - Google Sheets API エラー
- `GoogleAuthError` - 認証エラー
- `SpreadsheetNotFound` - スプレッドシート未検出
- `SheetNotFound` - シート未検出

### テーブル定義エラー

`BusinessRuleError` を使用：
- `TableDefinitionError` - テーブル定義不正

### ギルドID不一致エラー

`BusinessRuleError` を使用：
- `GuildIdMismatch` - ギルドID不一致

---

## エラーログ出力

### 構造化ログの実装

```rust
use tracing::{error, warn, info};

// Repository層エラー
pub fn log_repository_error(err: &RepositoryError, context: &str) {
    error!(
        target: "repository_error",
        error = %err,
        context = context,
        "データアクセスエラーが発生しました"
    );
}

// Service層エラー
pub fn log_validation_error(err: &ValidationError, user_id: i64) {
    warn!(
        target: "validation_error",
        error = %err,
        user_id = user_id,
        "バリデーションエラーが発生しました"
    );
}

pub fn log_business_error(err: &BusinessRuleError, context: &str) {
    warn!(
        target: "business_error",
        error = %err,
        context = context,
        "ビジネスルール違反が発生しました"
    );
}

pub fn log_external_service_error(err: &ExternalServiceError, service: &str) {
    error!(
        target: "external_service_error",
        error = %err,
        service = service,
        "外部サービスエラーが発生しました"
    );
}

// Facade層エラー
pub fn log_facade_error(err: &FacadeError, operation: &str) {
    error!(
        target: "facade_error",
        error = %err,
        operation = operation,
        "Facade層でエラーが発生しました"
    );
}
```

---

## エラーハンドリングパターン

### パターン1: トランザクション内でのエラーハンドリング

```rust
pub async fn execute_with_transaction<F, T>(
    db: &DatabaseConnection,
    operation: F,
) -> Result<T, FacadeError>
where
    F: FnOnce(&DatabaseTransaction) -> BoxFuture<'_, Result<T, FacadeError>>,
{
    let txn = db.begin().await.map_err(|e| FacadeError::TransactionError {
        message: format!("トランザクション開始失敗: {}", e),
    })?;

    let result = operation(&txn).await;

    match result {
        Ok(value) => {
            txn.commit().await.map_err(|e| FacadeError::TransactionError {
                message: format!("コミット失敗: {}", e),
            })?;
            Ok(value)
        }
        Err(e) => {
            txn.rollback().await.map_err(|rollback_err| {
                error!(
                    error = %rollback_err,
                    "ロールバック失敗"
                );
                FacadeError::TransactionError {
                    message: format!("ロールバック失敗: {}", rollback_err),
                }
            })?;
            Err(e)
        }
    }
}
```

### パターン2: 部分的エラー処理（データ変換）

```rust
pub fn convert_rows(
    rows: Vec<RawRow>,
) -> (Vec<ConvertedRow>, Vec<ValidationError>) {
    let mut converted = Vec::new();
    let mut errors = Vec::new();

    for (index, row) in rows.iter().enumerate() {
        match convert_row(row) {
            Ok(converted_row) => converted.push(converted_row),
            Err(e) => {
                warn!(
                    row_index = index,
                    error = %e,
                    "行の変換に失敗しました（スキップします）"
                );
                errors.push(e);
            }
        }
    }

    (converted, errors)
}
```

---

## テスト戦略

### エラー変換のテスト

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repository_error_to_business_error() {
        let repo_err = RepositoryError::NotFound {
            entity_type: "Quest".to_string(),
            id: "123".to_string(),
        };

        let business_err = BusinessRuleError::from(repo_err);
        assert!(matches!(business_err, BusinessRuleError::InvalidState { .. }));
    }

    #[test]
    fn test_validation_error_message() {
        let err = ValidationError::RequiredFieldMissing {
            field: "quest_name".to_string(),
        };

        let message = err.to_string();
        assert!(message.contains("必須フィールドが未入力"));
        assert!(message.contains("quest_name"));
    }

    #[test]
    fn test_presentation_error_conversion() {
        let validation_err = ValidationError::InvalidFormat {
            field: "date".to_string(),
            reason: "YYYY-MM-DD形式である必要があります".to_string(),
        };

        let facade_err = FacadeError::Validation {
            source: validation_err,
        };

        let presentation_err = PresentationError::from(facade_err);
        let message = presentation_err.to_string();
        assert!(message.contains("❌ 入力エラー"));
    }
}
```

---

## 実装時の注意点

### Do's（推奨事項）

1. **具体的なエラーメッセージ**: ユーザーが理解できる具体的なメッセージ
2. **エラーの分類**: エラーの原因や種類を適切に分類
3. **構造化ログ**: 検索・分析しやすい構造化ログを使用
4. **`#[from]`属性**: 自動変換を明示的に定義
5. **コンテキスト情報**: エラー発生時の状況情報を保持

### Don'ts（避けるべき事項）

1. **`panic!`の使用**: 回復不可能な状況以外では使用しない
2. **`unwrap()`の多用**: 本番コードでは避ける
3. **汎用的すぎるエラー**: `String`だけのエラーは避ける
4. **機密情報の漏洩**: エラーメッセージに機密情報を含めない
5. **ログの重複**: 同一エラーの多重ログ出力は避ける

---

## 関連ドキュメント

- [エラーハンドリングルール](../rules/error_handling.md)
- [ロギングルール](../rules/logging.md)
- [スプレッドシート機能設計](../features/google_spreadsheet.md)
