# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

GBF Discord Bot は、Granblue Fantasy（グラブル）のゲーム活動をサポートするDiscord Bot。Rust + poise + PostgreSQL + SeaORMで実装され、クリーンアーキテクチャを採用している。

## Common Commands

### Development
```bash
# Build the project
cargo build

# Build for release
cargo build --release

# Run the bot (requires .env configuration)
cargo run

# Run tests
cargo test

# Run a specific test
cargo test test_name

# Run with test logging enabled
cargo test -- --nocapture
```

### Linting and Formatting
```bash
# Check code with Clippy
cargo clippy

# Format code
cargo fmt

# Check formatting without modifying files
cargo fmt -- --check
```

### Database Migrations
```bash
# Run migrations
cargo run -- migrate

# Create a new migration (via migration crate)
cd migration
sea-orm-cli migrate generate migration_name
```

## Architecture

### Clean Architecture Layers

プロジェクトは明確な層別責務を持つクリーンアーキテクチャを採用：

```
events (Presentation) → facades (Application) → services (Business Logic) → repository (Data Access)
```

#### Layer Responsibilities

- **events/**: Discord イベント・インタラクション処理（スラッシュコマンド、ボタン、リアクション等）
  - Facadeを呼び出す（1対1関係）
  - Service層・Repository層への直接アクセスは禁止

- **facades/**: 複数サービスの協調、トランザクション境界管理
  - **トランザクション管理の唯一の責任層**（begin/commit/rollback）
  - Service層を組み合わせてユースケースを実現

- **services/**: 単一業務処理、ドメインルール実装
  - Repository層を呼び出し
  - トランザクションは引数で受け取り、Repository層に渡すのみ
  - 他Serviceへの直接依存は禁止

- **repository/**: データ永続化・取得の抽象化
  - トランザクションを使用したDB操作（`create_with_txn`等）
  - ビジネスロジック実装は禁止

### Dependency Injection

AppStateパターンを使用（DIコンテナではなくRustらしいアプローチ）：

- `main.rs`で単一のDB接続とAppStateを初期化
- PoiseDataを通じて全層に共有
- コンストラクタインジェクションで依存性を注入
- 各層での個別DB接続作成は禁止

### Transaction Management (重要)

**Facade層でのみトランザクション管理可能**：

```rust
// Facade層
let txn = app_state.db().begin().await?;
let result = async {
    service_layer_call(&txn).await?;  // Service層に渡す
    Ok(())
}.await;
match result {
    Ok(_) => txn.commit().await?,    // Facade層でコミット
    Err(e) => txn.rollback().await?, // Facade層でロールバック
}

// Service層
pub async fn service_function(txn: &DatabaseTransaction) -> Result<()> {
    repository.create_with_txn(txn, ...).await?;  // Repository層に渡す
    Ok(())  // commit/rollbackはしない
}

// Repository層
pub async fn create_with_txn(&self, txn: &DatabaseTransaction, ...) -> Result<()> {
    entity.insert(txn).await?;  // トランザクションを使用
    Ok(())  // commit/rollbackはしない
}
```

## Coding Standards

### Language

- **すべてのコメント、ドキュメント、エラーメッセージは日本語で記述**
- コード自体は英語（変数名、関数名等）

### Naming Conventions

- 構造体・列挙型・型エイリアス: `PascalCase`
- 関数・メソッド・変数: `snake_case`
- 定数: `SCREAMING_SNAKE_CASE`

### Error Handling

- `thiserror`を使用した構造化エラー定義
- 各層で適切なエラー型を定義（ValidationError, BusinessRuleError等）
- `#[from]`属性で層間エラー変換を実装
- 本番コードでの`unwrap()`使用禁止
- `panic!()`は回復不可能な状況のみ

### Logging

`tracing`クレートを使用した構造化ログ：

```rust
use tracing::{error, warn, info, debug};

// ERROR: システムエラー、予期しない例外
error!(error = %e, user_id = %user_id, "ユーザー作成に失敗しました");

// WARN: 業務例外、リトライ可能なエラー
warn!(recruitment_id = %id, "募集が満員のため参加を拒否しました");

// INFO: 重要な業務処理の開始・終了
info!(quest_name = %quest_name, "募集作成を開始しました");
```

## Important Constraints

### Database Rules

- トランザクション外でのDB操作禁止
- Service層・Repository層でのトランザクション生成・コミット・ロールバック禁止
- 長時間保持するトランザクション禁止

### Architecture Rules

- 層をまたいだ直接アクセス禁止（例: Facade → Repository直接呼び出し）
- グローバル変数の使用禁止
- 長すぎる関数（100行超）禁止
- 深すぎるネスト（5レベル超）禁止

### Performance Rules

- 不要な`clone()`を避け、借用を活用
- `Arc<T>`の多用を避け、必要最小限に
- 並行処理可能な箇所では`futures::future::try_join_all`等を活用

## Testing

### Test Structure

- 単体テスト: 各モジュール内に`#[cfg(test)]`
- 統合テスト: `tests/`ディレクトリ
- `mockall`を使用したモッキング
- Arrange-Act-Assert パターン推奨

### Running Tests

```rust
#[tokio::test]
async fn test_example() {
    // Arrange
    let mock = setup_mock();

    // Act
    let result = function_under_test(mock).await;

    // Assert
    assert!(result.is_ok());
}
```

## Key Technologies

- **Discord Bot Framework**: poise 0.6.1
- **Async Runtime**: tokio 1.47 (multi-thread)
- **ORM**: SeaORM 1.1 (PostgreSQL)
- **Error Handling**: thiserror 1.0
- **Logging**: tracing 0.1 + tracing-subscriber 0.3
- **Testing**: tokio-test, mockall

## Documentation Structure

設計書は`docs/develop/`に配置：

- `design/`: アーキテクチャ・機能設計書（抽象的な概念レベル）
- `rules/`: コーディングルール、開発ルール

**設計書の抽象化原則**：
- 具体的なコード実装を記載しない
- アーキテクチャ、責務、フローを概念的に記述
- コードと設計書の二重管理を回避