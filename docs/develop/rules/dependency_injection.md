# 依存性注入ルール

## 基本方針

- **コンストラクタインジェクション**: すべての依存関係はコンストラクタで受け取る
- **抽象化の活用**: traitによる抽象化を用いた依存関係の定義を推奨
- **シングルトン的DB接続**: DB接続プールは実質的にシングルトンとして機能するが、グローバル変数ではなく依存性注入によって管理

## 具体的実装指針

### コンストラクタインジェクションの詳細ルール

- **必須**: すべての依存関係はコンストラクタ（`new`メソッド）で受け取る
- **禁止**: `new()`メソッド内での他のオブジェクトの`new()`呼び出し
- **推奨**: traitによる抽象化を用いた依存関係の定義

**実装例**:

```rust
// ✅ 正しい実装
impl TransactionManager {
    pub fn new(db_service: Arc<dyn DatabaseService>, repos: RepositoryContainer) -> Self {
        Self { db_service, repos }
    }
}

// ❌ 間違った実装
impl TransactionManager {
    pub async fn new() -> Result<Self, PoiseError> {
        let db_manager = DatabaseConnectionManager::new().await?; // 禁止
        let repos = RepositoryContainer::new().await?; // 禁止
        // ...
    }
}
```

### 各層での依存関係受け取りパターン

#### プレゼンテーション層（Events/Commands）

- `Context`または`Data`から必要な依存関係を取得
- Facade層への依存関係注入を行う

```rust
pub async fn handle_command(ctx: PoiseContext) -> Result<(), PoiseError> {
    let tx_manager = ctx.data().transaction_manager.clone();
    let facade = BattleRecruitmentFacade::new(tx_manager);
    facade.create_new_recruitment(/* params */).await
}
```

#### Facade層

- Service層とTransactionManagerを依存関係として受け取る
- 1つのオペレーションを担当するクラスとして実装

```rust
pub struct BattleRecruitmentFacade {
    tx_manager: Arc<TransactionManager>,
    new_service: Arc<dyn NewRecruitmentService>,
    update_service: Arc<dyn UpdateRecruitmentService>,
}

impl BattleRecruitmentFacade {
    pub fn new(
        tx_manager: Arc<TransactionManager>,
        new_service: Arc<dyn NewRecruitmentService>,
        update_service: Arc<dyn UpdateRecruitmentService>,
    ) -> Self {
        Self { tx_manager, new_service, update_service }
    }
}
```

#### Service層

- Repository層の依存関係を受け取る
- 単一責任の原則に従う

```rust
pub struct NewRecruitmentService {
    repo: Arc<dyn BattleRecruitmentRepository>,
}

impl NewRecruitmentService {
    pub fn new(repo: Arc<dyn BattleRecruitmentRepository>) -> Self {
        Self { repo }
    }
}
```

## DB接続管理とDI

### DB接続管理の基本原則

- **main関数でのDB接続初期化**: アプリケーション起動時（main関数実行～bot起動まで）に単一のDB接続（コネクションプール含む）を作成し、これを全体で共有する
- **依存性注入による配布**: 作成したDB接続は依存性注入（Dependency Injection）パターンを使って各層に配布する
- **シングルトン化**: DB接続は実質的にシングルトンとして機能するが、グローバル変数ではなく依存性注入によって管理する
- **データアクセス層での使用制限**: DB接続の直接的な操作はデータアクセス層（infrastructure, repository）でのみ許可される

### 依存性注入の流れ

1. **main.rs**: 単一のDB接続を作成
2. **プレゼンテーション層**: 共有DB接続から必要な依存関係オブジェクトを準備
3. **Facade層**: プレゼンテーション層から依存関係を受け取り、Service層に渡す
4. **Service層**: Repository層への依存関係を管理
5. **Repository層**: 注入されたDB接続を使用して実際の操作を実行

### コンストラクタでの依存関係受け取り

- **TransactionManager**: `new(db_service, repos)` - DB接続関連オブジェクトを外部から受け取る
- **RepositoryContainer**: `new_with_connection(db_connection)` - 共有DB接続を外部から受け取る
- **各Repository実装**: `new(db_connection)` - 共有DB接続を外部から受け取る

## 禁止事項

### 全般的な禁止事項

- 各層での個別DB接続作成は禁止（`DatabaseConnectionManager::new()`の直接呼び出し禁止）
- Repository層でのDB接続引数受け取りは禁止（トランザクションのみ受け取る）
- サービスロケータパターンによるグローバル状態への依存は禁止

### 具体的な禁止パターン

```rust
// ❌ 禁止: コンストラクタ内での他オブジェクト生成
impl SomeService {
    pub fn new() -> Self {
        let repo = SomeRepository::new(); // 禁止
        Self { repo }
    }
}

// ❌ 禁止: グローバル状態への依存
lazy_static! {
  static ref DB_CONNECTION: DatabaseConnection = create_connection();
}

// ❌ 禁止: Repository層でのDB接続直接受け取り
impl SomeRepository {
    pub fn save(&self, data: Data, db_conn: &DatabaseConnection) -> Result<()> {
        // 禁止: DB接続を引数で受け取る
    }
}
```

## 推奨パターン

### 依存関係の抽象化

```rust
// ✅ 推奨: traitによる抽象化
pub trait UserRepository {
    async fn find_by_id(&self, id: UserId, tx: &Transaction) -> Result<Option<User>>;
    async fn save(&self, user: &User, tx: &Transaction) -> Result<()>;
}

pub struct UserService {
    user_repo: Arc<dyn UserRepository>,
}

impl UserService {
    pub fn new(user_repo: Arc<dyn UserRepository>) -> Self {
        Self { user_repo }
    }
}
```

### ライフサイクル管理

```rust
// ✅ 推奨: 適切なライフサイクル管理
pub struct AppState {
    pub transaction_manager: Arc<TransactionManager>,
    pub repositories: RepositoryContainer,
}

impl AppState {
    pub async fn new() -> Result<Self> {
        let db_connection = DatabaseConnectionManager::new().await?;
        let repositories = RepositoryContainer::new_with_connection(db_connection.clone());
        let transaction_manager = Arc::new(TransactionManager::new(db_connection, repositories.clone()));

        Ok(Self {
            transaction_manager,
            repositories,
        })
    }
}
```