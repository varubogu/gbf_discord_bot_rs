# コーディングスタイル規約

## 基本方針

- **Rustらしいコーディング**: Rustのイディオムとベストプラクティスに従う
- **一貫性の重視**: プロジェクト全体で統一されたスタイルを維持
- **可読性の優先**: コードの理解しやすさを最優先
- **日本語対応**: コメント・ドキュメント・エラーメッセージは日本語で記述
- **プロジェクト固有パターンの遵守**: アーキテクチャルールに準拠した実装

## 命名規則

### 基本命名規則

#### モジュール・ファイル名

```rust
// ✅ 正しい
mod battle_recruitment;  // snake_case
mod user_service;       // snake_case
mod quest_start;        // snake_case

// ❌ 間違い
mod BattleRecruitment;  // PascalCase禁止
mod userService;        // camelCase禁止
```

#### 構造体・列挙型・型エイリアス

```rust
// ✅ 正しい
pub struct BattleRecruitmentFacade;    // PascalCase
pub enum ValidationError;              // PascalCase
pub type Result<T> = std::result::Result<T, AppError>;  // PascalCase

// ❌ 間違い
pub struct battle_recruitment_facade;  // snake_case禁止
pub enum validation_error;             // snake_case禁止
```

#### 関数・メソッド・変数

```rust
// ✅ 正しい
pub fn start_recruitment() {}          // snake_case
pub async fn create_new_recruitment() {} // snake_case
let user_id = 123;                     // snake_case
let max_participants = 4;              // snake_case

// ❌ 間違い
pub fn StartRecruitment() {}           // PascalCase禁止
pub fn createNewRecruitment() {}       // camelCase禁止
```

#### 定数

```rust
// ✅ 正しい
pub const DEFAULT_MAX_PARTICIPANTS: u32 = 4;  // SCREAMING_SNAKE_CASE
const DB_CONNECTION_TIMEOUT: Duration = Duration::from_secs(30);

// ❌ 間違い
const default_max_participants: u32 = 4;      // snake_case禁止
```

#### トレイト名

```rust
// ✅ 正しい
pub trait BattleRecruitmentRepository {}       // PascalCase
pub trait DatabaseService {}                  // PascalCase

// 実装型には明示的にImplサフィックス
pub struct BattleRecruitmentRepositoryImpl {} // PascalCase + Impl
```

### プロジェクト固有命名規則

#### Facade層

```rust
// ✅ 推奨パターン
pub struct BattleRecruitmentFacade;
pub struct UserSettingsFacade;
pub struct SchedulerFacade;
```

#### Service層

```rust
// ✅ 推奨パターン
pub struct StartRecruitmentService;
pub struct NewRecruitmentService;
pub struct UserValidationService;
```

#### Repository層

```rust
// ✅ 推奨パターン
pub trait BattleRecruitmentRepository {}
pub struct BattleRecruitmentRepositoryImpl;
pub trait UserRepository {}
pub struct UserRepositoryImpl;
```

## コードフォーマット

### インデント・改行

```rust
// ✅ 正しい: 4スペースインデント
impl BattleRecruitmentFacade {
    pub fn new(
        tx_manager: Arc<TransactionManager>,
        service: Arc<dyn NewRecruitmentService>,
    ) -> Self {
        Self {
            tx_manager,
            service,
        }
    }
}

// ✅ 正しい: 長い引数は改行して整理
pub async fn create_recruitment(
    &self,
    guild_id: u64,
    channel_id: u64,
    quest_name: &str,
    max_participants: u32,
) -> Result<Recruitment> {
    // 処理内容
}
```

### 括弧・空白

```rust
// ✅ 正しい
if condition {
// 処理
}

match result {
Ok(value) => value,
Err(e) => return Err(e),
}

let result = some_function(param1, param2);
```

### 行長制限

### 文字列フォーマット（format!マクロ）

- **禁止**: `format!("{}", a)` のようにフォーマット文字列と引数を分離する書き方は使用禁止
- **推奨**: 変数埋め込みを使用して `format!("{a}")` の形式で記述すること
- **理由**: 可読性向上と、`cargo clippy` による警告検出の回避のため

```rust
// ❌ 禁止
let s = format!("{}", user_name);

// ✅ 推奨
let s = format!("{user_name}");
```
- 長い行は適切な位置で改行する

## import文の整理

### import順序

```rust
// 1. 標準ライブラリ
use std::sync::Arc;
use std::pin::Pin;
use std::time::Duration;

// 2. 外部クレート（アルファベット順）
use sea_orm::{TransactionTrait, DatabaseConnection};
use tracing::{error, info, instrument};
use uuid::Uuid;

// 3. 内部モジュール（階層順）
use crate::types::{AppError, Result, PoiseContext};
use crate::services::battle_recruitment::start::StartRecruitmentService;
use crate::repository::database::battle_recruitment_repository::BattleRecruitmentRepositoryImpl;
```

### import文のグループ化

```rust
// ✅ 正しい: 空行でグループ分け
use std::sync::Arc;
use std::pin::Pin;

use sea_orm::TransactionTrait;
use tracing::{error, info};

use crate::types::Result;
use crate::services::battle_recruitment;

// ❌ 間違い: グループ分けなし
use std::sync::Arc;
use sea_orm::TransactionTrait;
use crate::types::Result;
use std::pin::Pin;
```

### asエイリアスの使用

```rust
// ✅ 推奨: 名前衝突回避時
use crate::facades::recruit as recruit_facade;
use serenity::model::channel::Message as DiscordMessage;

// ✅ 推奨: 長いパス短縮時
use crate::repository::database::battle_recruitment_repository as br_repo;
```

## コメント・ドキュメント規約

### ドキュメントコメント

```rust
/// 募集を開始する（クロージャパターン）
///
/// # Arguments
/// * `ctx` - Poiseコンテキスト
/// * `guild_id` - ギルドID
/// * `channel_id` - チャンネルID
/// * `message_id` - メッセージID
///
/// # Returns
/// 処理結果（成功時はUnit、失敗時はAppError）
///
/// # Examples
/// ```rust
/// let result = start_recruitment(ctx, guild_id, channel_id, message_id).await?;
/// ```
#[instrument]
pub async fn start_recruitment(
    ctx: PoiseContext<'_>,
    guild_id: u64,
    channel_id: u64,
    message_id: u64,
) -> Result<()> {
    // 実装
}
```

### インラインコメント

```rust
// ✅ 正しい: 日本語での説明
pub async fn create_recruitment(&self, data: CreateData) -> Result<Recruitment> {
    info!("募集作成を開始しました");

    // Repository作成（Service層への依存性注入のため）
    let repo = BattleRecruitmentRepositoryImpl::new(self.db.clone());

    // Service層経由で作成処理
    let service = NewRecruitmentService::new(Arc::new(repo));
    service.create(data).await
}
```

### TODO・FIXME・NOTE

```rust
// TODO: パフォーマンス最適化が必要
// FIXME: エラーハンドリングの改善
// NOTE: この処理は仕様変更により将来削除予定
```

## エラーハンドリングスタイル

### エラー型定義

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
```

### エラーメッセージ

- **日本語**: すべてのエラーメッセージは日本語で記述
- **具体性**: 具体的で分かりやすい内容
- **一貫性**: 同じ種類のエラーは統一された表現

### エラー変換

```rust
// ✅ 推奨: #[from]属性による自動変換
#[derive(Error, Debug)]
pub enum DataAccessError {
    #[error("データベース接続エラー: {source}")]
    ConnectionError {
        #[from]
        source: sea_orm::DbErr
    },
}
```

## 依存性注入パターン

### コンストラクタ設計

```rust
// ✅ 正しい: 依存関係を外部から受け取る
impl BattleRecruitmentFacade {
    pub fn new(
        tx_manager: Arc<TransactionManager>,
        service: Arc<dyn NewRecruitmentService>,
    ) -> Self {
        Self { tx_manager, service }
    }
}

// ❌ 間違い: コンストラクタ内での依存関係生成
impl BattleRecruitmentFacade {
    pub fn new() -> Self {
        let service = NewRecruitmentService::new(); // 禁止
        Self { service }
    }
}
```

### トレイト抽象化

```rust
// ✅ 推奨: Arc<dyn Trait>パターン
pub struct UserService {
    user_repo: Arc<dyn UserRepository>,
    validator: Arc<dyn UserValidator>,
}

impl UserService {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        validator: Arc<dyn UserValidator>,
    ) -> Self {
        Self { user_repo, validator }
    }
}
```

## ログ出力スタイル

### ログレベルの使い分け

```rust
use tracing::{error, warn, info, debug, trace};

// ERROR: システムエラー、予期しない例外
error!(error = %e, user_id = %user_id, "ユーザー作成に失敗しました");

// WARN: 業務例外、リトライ可能なエラー
warn!(recruitment_id = %id, "募集が満員のため参加を拒否しました");

// INFO: 重要な業務処理の開始・終了
info!(quest_name = %quest_name, "募集作成を開始しました");

// DEBUG: 詳細なトレース情報
debug!(participant_count = count, "現在の参加者数");

// TRACE: より詳細なトレース情報
trace!(sql = %query, "SQL実行");
```

### 構造化ログフォーマット

```rust
// ✅ 推奨: フィールド付きログ
info!(
    recruitment_id = %recruitment.id(),
    quest_name = %recruitment.quest_name(),
    participant_count = recruitment.participants().len(),
    "募集作成が完了しました"
);

// ❌ 非推奨: 文字列埋め込み
info!("募集作成が完了しました: ID={}", recruitment.id());
```

### instrument属性

```rust
// ✅ 推奨: 関数トレース用
#[instrument]
pub async fn start_recruitment(
    ctx: PoiseContext<'_>,
    guild_id: u64,
    channel_id: u64,
) -> Result<()> {
    // 処理内容
}

// ✅ 推奨: フィールド指定
#[instrument(skip(ctx), fields(guild_id = %guild_id))]
pub async fn complex_function(
    ctx: PoiseContext<'_>,
    guild_id: u64,
) -> Result<()> {
    // 処理内容
}
```

## 非同期処理スタイル

### async/await

```rust
// ✅ 正しい
pub async fn create_recruitment(&self) -> Result<Recruitment> {
    let data = self.validate_input().await?;
    let recruitment = self.repository.save(&data).await?;
    self.notify_participants(&recruitment).await?;
    Ok(recruitment)
}

// ✅ 正しい: エラーハンドリング付き
pub async fn process_with_retry(&self) -> Result<()> {
    match self.try_operation().await {
        Ok(result) => Ok(result),
        Err(e) => {
            warn!(error = %e, "操作に失敗、リトライします");
            self.try_operation().await
        }
    }
}
```

## テスト記述方式

### テスト関数命名

```rust
// ✅ 推奨: 日本語での説明的命名
#[tokio::test]
async fn 募集作成_正常系_成功する() {
    // テスト実装
}

#[tokio::test]
async fn 募集作成_参加者数上限超過_エラーになる() {
    // テスト実装
}

// ✅ 代替案: 英語での説明的命名
#[tokio::test]
async fn create_recruitment_with_valid_data_succeeds() {
    // テスト実装
}
```

### テスト構成

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn 募集作成_正常系_成功する() {
        // Arrange
        let facade = create_test_facade();
        let data = create_valid_recruitment_data();

        // Act
        let result = facade.create_recruitment(data).await;

        // Assert
        assert!(result.is_ok());
        let recruitment = result.unwrap();
        assert_eq!(recruitment.quest_name(), "テストクエスト");
    }
}
```

## モジュール構成

### mod.rs設計

```rust
// src/facades/mod.rs
pub mod recruit;
pub mod settings;
pub mod scheduler;

// 再エクスポート（必要に応じて）
pub use recruit::*;
pub use settings::*;
```

### pub使用方針

```rust
// ✅ 正しい: 必要最小限のpub
pub struct PublicStruct;          // 外部公開が必要
struct PrivateStruct;             // モジュール内部のみ

pub fn public_function() {}       // 外部公開が必要
fn private_function() {}          // モジュール内部のみ

// ✅ 正しい: 条件付きpub
pub(crate) struct CrateInternalStruct;  // クレート内のみ
pub(super) fn parent_only_function() {} // 親モジュールのみ
```

## 属性・マクロ使用規則

### 推奨属性

```rust
// デッドコード警告抑制
#[allow(dead_code)]
async fn future_feature() {}

// 関数トレース
#[instrument]
pub async fn important_function() {}

// 構造体デバッグ
#[derive(Debug, Clone)]
pub struct ImportantStruct;

// エラー型
#[derive(Error, Debug)]
pub enum CustomError {}
```

### 条件付きコンパイル

```rust
// テストコード
#[cfg(test)]
mod tests {}

// 開発環境のみの機能
#[cfg(debug_assertions)]
fn debug_only_function() {}
```

## 禁止事項

### 全般的禁止事項

- グローバル変数の使用（`lazy_static!`、`static`の濫用）
- `unwrap()`の本番コードでの使用（テストは除く）
- `panic!()`の使用（回復不可能な状況を除く）
- 長すぎる関数（100行超）
- 深すぎるネスト（5レベル超）

### プロジェクト固有禁止事項

```rust
// ❌ 禁止: 直接DB接続作成
let db = Database::connect("...").await?;

// ❌ 禁止: Repository層でのDB接続引数受け取り
fn save(&self, data: Data, db: &DatabaseConnection) {}

// ❌ 禁止: unwrapの濫用
let value = some_option.unwrap(); // 本番コードでは禁止

// ❌ 禁止: エラーメッセージの英語
#[error("User not found")]  // 日本語で記述すること
```

## rustfmtとClippy設定

### rustfmt設定（rust.toml）

```toml
# 推奨設定
max_width = 100
hard_tabs = false
tab_spaces = 4
newline_style = "Unix"
use_small_heuristics = "Default"
reorder_imports = true
reorder_modules = true
remove_nested_parens = true
```

### Clippy警告レベル

```toml
# Cargo.toml
[lints.clippy]
all = "warn"
pedantic = "warn"
nursery = "warn"
```

## まとめ

このコーディングスタイル規約は、プロジェクトの品質と保守性を向上させるための指針です。

**重要原則**:

1. **一貫性**: プロジェクト全体で統一されたスタイル
2. **可読性**: 理解しやすいコードを優先
3. **日本語対応**: ユーザー向けメッセージは日本語
4. **型安全性**: Rustの型システムを最大限活用
5. **アーキテクチャ遵守**: 定義されたアーキテクチャルールに従う

規約は開発チーム全体で遵守し、コードレビュー時に積極的にチェックすることで、保守しやすく品質の高いコードベースを維持します。