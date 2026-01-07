# 依存性注入（DI）設計書

> **⚠️ 注意: これは設計提案ドキュメントです**
>
> このドキュメントに記載されている `DIContainer` パターンは設計提案であり、**実際には実装されていません**。
>
> 現在の実装では、`AppState` パターンを採用しており、以下の構造になっています:
> - `AppState` 構造体がアプリケーションの共有状態を保持
> - 3つのDB接続（`guild_db`, `system_db`, `global_db`）とサービス（`MessageService`）を管理
> - `PoiseData` 構造体内に `AppState` を格納し、全コマンドからアクセス可能
>
> 実装の詳細は [rust_optimized_architecture.md](rust_optimized_architecture.md) を参照してください。

## 1. 概要

本設計書では、Discord Bot（Rust + poise）における依存性注入（Dependency Injection）パターンの設計方針について詳述します。

### 1.1 作成日時

2025-08-22

### 1.2 目的

- テスタビリティとメンテナンス性の向上
- クリーンアーキテクチャの原則遵守
- 依存関係の明確化と管理の改善

## 2. 現状分析

### 2.1 現在のアーキテクチャの問題点

#### 問題1: Factory Patternによる依存関係の自己作成

各オブジェクトが自分で依存関係を作成する構造になっており、依存関係が隠蔽されています。

**影響:**

- テストでのモッキングが困難
- 依存関係の変更時の影響範囲が不明確
- デバッグ時の問題特定困難

#### 問題3: PoiseDataの活用不足

```rust
// src/types/mod.rs
pub struct PoiseData {}  // 空の構造体
```

Discord Botフレームワーク（poise）が提供する共有データ機構が活用されていません。

### 2.2 パフォーマンスへの影響

1. **メモリ使用量増加**: 複数のコネクションプールによる不要なメモリ消費
2. **DB接続数増加**: 各プール毎の最小接続数確保
3. **初期化時間延長**: 複数回のDB接続処理

## 3. 解決方針：依存性注入パターンの採用

### 3.1 依存性注入とは

依存性注入（Dependency Injection）は、オブジェクトが自分で依存関係を作成するのではなく、外部から注入されるパターンです。

#### 従来のFactory Pattern

```rust
impl TransactionManager {
    pub async fn new() -> Result<Self, PoiseError> {
        let db_manager = DatabaseConnectionManager::new().await?;  // 自己作成
        // ...
    }
}
```

#### 依存性注入パターン

```rust
impl TransactionManager {
    pub fn new(db_service: Arc<SeaOrmDatabase>, repos: Arc<RepositoryContainer>) -> Self {
        Self { db_service, repos }  // 外部から注入
    }
}
```

### 3.2 DIパターンの利点

1. **テスタビリティ**: モックオブジェクトの注入が容易
2. **明示的な依存関係**: コンストラクタシグネチャで依存関係が明確
3. **リソース効率**: 単一のコネクションプールを共有
4. **保守性**: 依存関係の変更が局所化
5. **デバッグ容易性**: 依存関係の追跡が簡単

## 4. 設計方針

### 4.1 アーキテクチャ方針

#### 4.1.1 単一DB接続の原則

- `main.rs`で単一のDB接続（コネクションプール含む）を作成
- 全てのRepository層はこの共有接続を利用
- 個別DB接続作成は禁止

#### 4.1.2 依存性注入の流れ

```
main.rs → プレゼンテーション層 → Facade層 → Service層 → Repository層
```

#### 4.1.3 層別責務

- **main.rs**: DB接続の初期化と共有インスタンス作成
- **プレゼンテーション層**: 共有インスタンスから依存関係を準備
- **Facade層**: 依存関係を受け取り、Service層に注入
- **Service層**: Repository層への依存関係を管理
- **Repository層**: 注入されたDB接続を使用

### 4.2 技術的方針

#### 4.2.1 コンストラクタインジェクション

全ての依存関係はコンストラクタ（`new`メソッド）で受け取る

#### 4.2.2 Arc<T>による共有

`Arc<T>`を使用して複数の参照先での安全な共有を管理

#### 4.2.3 トレイトベースのインターフェース

テスタビリティ向上のため、具象型ではなくトレイトを依存関係とする

## 5. 具体的設計

### 5.1 main.rsでの初期化

```rust
#[tokio::main]
async fn main() {
    // 環境設定...
    
    // 単一のDB接続を作成（コネクションプール含む）
    let database_url = build_database_url().expect("Failed to build database URL");
    let db_connection = SeaDatabase::connect(&database_url)
        .await
        .expect("Failed to connect to database");
    
    // 共有DB接続を使用してサービス群を初期化
    let db_service = Arc::new(SeaOrmDatabase::new(db_connection.clone()));
    let repos = Arc::new(RepositoryContainer::new_with_connection(db_connection));
    
    // Dependency Injection Container（DIコンテナ）の作成
    let di_container = Arc::new(DIContainer::new(db_service, repos));
    
    // Framework作成...
    let framework = poise::Framework::builder()
        .setup(move |ctx, _ready, framework| {
            let di_container = di_container.clone();
            Box::pin(async move {
                // DIコンテナをPoiseDataに格納
                let data = PoiseData { di_container };
                Ok(data)
            })
        })
        .build();
}
```

### 5.2 DIコンテナの設計

```rust
// src/infrastructure/di_container.rs
pub struct DIContainer {
    db_service: Arc<SeaOrmDatabase>,
    repos: Arc<RepositoryContainer>,
}

impl DIContainer {
    pub fn new(db_service: Arc<SeaOrmDatabase>, repos: Arc<RepositoryContainer>) -> Self {
        Self { db_service, repos }
    }
    
    pub fn get_db_service(&self) -> Arc<SeaOrmDatabase> {
        self.db_service.clone()
    }
    
    pub fn get_repos(&self) -> Arc<RepositoryContainer> {
        self.repos.clone()
    }
    
    pub fn create_transaction_manager(&self) -> TransactionManager {
        TransactionManager::new(self.db_service.clone(), self.repos.clone())
    }
}
```

### 5.3 PoiseDataの拡張

```rust
// src/types/mod.rs
#[derive(Debug)]
pub struct PoiseData {
    pub di_container: Arc<DIContainer>,
}
```

### 5.4 Repository層の設計変更

#### 5.4.1 RepositoryContainer

```rust
// src/infrastructure/database/container.rs
impl RepositoryContainer {
    // 依存性注入対応のコンストラクタ
    pub fn new_with_connection(db_connection: DatabaseConnection) -> Self {
        let battle_recruitment_repo = Arc::new(BattleRecruitmentRepositoryImpl::new(
            db_connection.clone(),
        ));
        
        Self {
            battle_recruitment_repo,
        }
    }
    
    // 古いnew()メソッドは削除予定
    #[deprecated(note = "Use new_with_connection instead")]
    pub async fn new() -> Result<Self, PoiseError> {
        // 既存の実装...
    }
}
```

#### 5.4.2 TransactionManager

```rust
// src/infrastructure/database/transaction_manager.rs
impl TransactionManager {
    // 依存性注入対応のコンストラクタ
    pub fn new(db_service: Arc<SeaOrmDatabase>, repos: Arc<RepositoryContainer>) -> Self {
        Self { 
            db_service: (*db_service).clone(),
            repos: (*repos).clone(),
        }
    }
    
    // 古いnew()メソッドは削除予定
    #[deprecated(note = "Use dependency injection constructor instead")]
    pub async fn new() -> Result<Self, PoiseError> {
        // 既存の実装...
    }
}
```

### 5.5 プレゼンテーション層での利用

```rust
// src/events/interactions/command_interactions/slash/battle_recruitment.rs
#[poise::command(slash_command)]
pub async fn recruit(
    ctx: Context<'_>,
    // 他のパラメータ...
) -> Result<(), PoiseError> {
    // DIコンテナから依存関係を取得
    let di_container = &ctx.data().di_container;
    let transaction_manager = di_container.create_transaction_manager();
    
    // Facade層に注入
    let facade = BattleRecruitmentFacade::new(transaction_manager);
    
    // ビジネスロジック実行
    facade.create_recruitment(/* parameters */).await
}
```

### 5.6 Facade層の設計変更

```rust
// src/facades/battle_recruitment.rs
pub struct BattleRecruitmentFacade {
    transaction_manager: TransactionManager,
}

impl BattleRecruitmentFacade {
    // 依存性注入対応のコンストラクタ
    pub fn new(transaction_manager: TransactionManager) -> Self {
        Self { transaction_manager }
    }
    
    // ビジネスロジックメソッド...
    pub async fn create_recruitment(&self, /* parameters */) -> Result<(), PoiseError> {
        self.transaction_manager.execute_in_transaction(|ctx| {
            Box::pin(async move {
                // Repository層の利用
                ctx.repos.battle_recruitment_repo.create_with_txn(
                    ctx.txn, 
                    /* parameters */
                ).await
            })
        }).await
    }
}
```

## 6. 運用上の補足

依存性注入の導入は、単一接続共有と責務分離を守ることを前提としています。導入時は Facade 層でのトランザクション境界と Service 層の依存受け渡しを維持し、既存実装から段階的に切り替える際も同じ原則のもとで整合性を確認してください。

## 7. 結論

依存性注入パターンの採用により、以下の改善が期待できます：

1. **リソース効率化**: 単一コネクションプールによる大幅なメモリ・接続数削減
2. **保守性向上**: 明示的な依存関係による理解しやすいコード
3. **テスタビリティ向上**: モックオブジェクト注入による包括的なテスト
4. **拡張性確保**: 将来的な機能拡張への対応力

現在のアーキテクチャ問題を解決し、長期的な保守性とパフォーマンスの両立を実現する最適な解決策と判断します。

---

**更新履歴**:

- 2025-08-22: 初版作成
- 2025-10-21: 運用情報の整理と設計ガイドラインの明確化
