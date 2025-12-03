# データベース接続・トランザクション管理設計書

## 1. 概要

本設計書では、Discord Bot（Rust + poise + SeaORM）におけるデータベース接続とトランザクション管理の現在のアーキテクチャについて詳述します。

### 1.1 作成日時

2025-08-24

### 1.2 目的

- ORM非依存のトランザクション抽象化の実現
- TransactionManagerによる統一的なトランザクション管理
- クリーンアーキテクチャの原則に従ったDB接続管理
- 将来的な他のORM（Diesel、SQLx等）への切り替え容易性

## 2. アーキテクチャ概要

### 2.1 設計原則

#### ORM非依存性

- 抽象化トレイト(`DatabaseTransactionTrait`、`DatabaseConnectionTrait`)による依存性逆転
- SeaORM固有の実装を抽象化層で隠蔽
- 将来的な他のORM対応の容易性

#### レイヤー間の責務分離

```
Facade層 → TransactionManager → SeaOrmTransaction → Repository層
```

- Facade層: TransactionManager経由でのみRepository層にアクセス
- TransactionManager: トランザクションの開始・コミット・ロールバック管理
- Repository層: トランザクション対応メソッドでDB操作実行

#### AppStateパターン

- 単一のDB接続をAppStateで管理
- 複数接続作成の回避
- リソース効率的な共有状態管理

## 3. コンポーネント設計

### 3.1 トランザクション抽象化トレイト

**実装場所**: `src/types/transaction.rs`

#### DatabaseTransactionTrait

```rust
#[async_trait]
pub trait DatabaseTransactionTrait: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    /// トランザクションをコミットする
    async fn commit(self) -> Result<(), Self::Error>;

    /// トランザクションをロールバックする
    async fn rollback(self) -> Result<(), Self::Error>;
}
```

#### DatabaseConnectionTrait

```rust
#[async_trait]
pub trait DatabaseConnectionTrait: Send + Sync {
    type Transaction: DatabaseTransactionTrait;
    type Error: std::error::Error + Send + Sync + 'static;

    /// トランザクションを開始する
    async fn begin_transaction(&self) -> Result<Self::Transaction, Self::Error>;
}
```

**利点:**

- ORM固有の型への直接依存を排除
- 将来的な他のORM実装への切り替えが容易
- テスト時のモック実装が可能

### 3.2 SeaORM実装

**実装場所**: `src/infrastructure/database/sea_orm_transaction.rs`

#### SeaOrmTransaction

```rust
pub struct SeaOrmTransaction {
    inner: DatabaseTransaction,
}

impl DatabaseTransactionTrait for SeaOrmTransaction {
    type Error = sea_orm::DbErr;

    async fn commit(self) -> Result<(), Self::Error> {
        self.inner.commit().await
    }

    async fn rollback(self) -> Result<(), Self::Error> {
        self.inner.rollback().await
    }
}
```

#### SeaOrmConnection

```rust
pub struct SeaOrmConnection {
    inner: DatabaseConnection,
}

impl DatabaseConnectionTrait for SeaOrmConnection {
    type Transaction = SeaOrmTransaction;
    type Error = sea_orm::DbErr;

    async fn begin_transaction(&self) -> Result<Self::Transaction, Self::Error> {
        let txn = self.inner.begin().await?;
        Ok(SeaOrmTransaction::new(txn))
    }
}
```

**責務:**

- 抽象化トレイトの具体実装
- SeaORMの`DatabaseTransaction`をラップ
- AppError変換の実装

### 3.3 TransactionManager

**実装場所**: `src/infrastructure/database/transaction_manager.rs`

#### 設計思想

- **実用性重視**: 完全な抽象化よりもSeaORM特化の実装で簡潔性を保持
- **Facade層専用**: Facade層からのみ使用されることを想定
- **依存性注入対応**: DB接続を外部から注入

#### TransactionContext

```rust
pub struct TransactionContext<'a> {
    pub txn: &'a SeaOrmTransaction,
    pub repos: &'a RepositoryContainer,
}

impl<'a> TransactionContext<'a> {
    /// Repository層が使用するSeaORMトランザクションへのアクセス
    pub fn sea_orm_txn(&self) -> &sea_orm::DatabaseTransaction {
        self.txn.inner()
    }
}
```

#### TransactionManager本体

```rust
pub struct TransactionManager {
    db_connection: Arc<DatabaseConnection>,
    repos: RepositoryContainer,
}

impl TransactionManager {
    /// 依存性注入対応のコンストラクタ（推奨）
    pub fn new(db_connection: Arc<DatabaseConnection>) -> Self {
        let repos = RepositoryContainer::new(&db_connection);
        Self { db_connection, repos }
    }

    /// AppStateから作成するファクトリメソッド
    pub fn from_app_state(app_state: &AppState) -> Self {
        Self::new(app_state.db_connection.clone())
    }

    /// Facade専用：トランザクション内で処理を実行
    pub async fn execute_in_transaction<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(TransactionContext) -> Pin<Box<dyn Future<Output=Result<T>> + Send>> + Send,
        T: Send,
    {
        let sea_orm_txn = self.db_connection.begin().await?;
        let wrapped_txn = SeaOrmTransaction::new(sea_orm_txn);

        let result = {
            let ctx = TransactionContext::new(&wrapped_txn, &self.repos);
            f(ctx).await
        };

        match result {
            Ok(value) => {
                wrapped_txn.commit().await?;
                Ok(value)
            }
            Err(e) => {
                let _ = wrapped_txn.rollback().await; // ロールバックエラーは無視
                Err(e)
            }
        }
    }
}
```

**特徴:**

- 自動的なトランザクション管理（開始・コミット・ロールバック）
- Repository群への統一アクセス
- エラー時の確実なロールバック

## 4. ファイル構成

### 4.1 現在の構成

```
src/
├── types/
│   └── transaction.rs              # DB非依存の抽象トレイト
├── infrastructure/
│   └── database/
│       ├── connection/
│       │   ├── connection_manager.rs    # 環境変数→URL変換
│       │   └── sea_orm_connection.rs    # SeaORM接続実装
│       ├── container.rs            # RepositoryContainer
│       ├── sea_orm_transaction.rs  # SeaORM実装
│       └── transaction_manager.rs  # ORM非依存Manager
└── repository/
    └── database/
        └── battle_recruitment_repository.rs  # トランザクション対応メソッド
```

### 4.2 責務分担

#### types/transaction.rs

- DB非依存の抽象トレイト定義
- 他のORM実装のためのインターface提供

#### infrastructure/database/connection/

- 環境変数からDB URL構築（connection_manager.rs）
- SeaORM接続の抽象化実装（sea_orm_connection.rs）

#### infrastructure/database/sea_orm_transaction.rs

- SeaORM固有のトランザクション・接続実装
- 抽象トレイトの具体実装

#### infrastructure/database/transaction_manager.rs

- Facade層向けのトランザクション管理
- Repository群への統一アクセス

## 5. 使用パターンとベストプラクティス

### 5.1 Facade層での使用例

```rust
impl<'a> BattleRecruitmentFacade<'a> {
    pub async fn new_recruitment<F>(
        &self,
        quest_alias: &str,
        battle_type: BattleType,
        channel_id: u64,
        guild_id: u64,
        mut discord_operation: F,
    ) -> Result<u64>
    where
        F: FnMut(DiscordOperation) -> Pin<Box<dyn Future<Output=Result<DiscordOperationResult>> + Send>>,
    {
        let tx_manager = TransactionManager::from_app_state(self.app_state);

        tx_manager.execute_in_transaction(|tx_ctx| {
            Box::pin(async move {
                // Repository層へのアクセス
                let battle_recruitment_repo = tx_ctx.repos.battle_recruitment();

                // Discord操作（副作用の外部委譲）
                let discord_result = discord_operation(DiscordOperation::SendMessage {
                    channel_id,
                    content: message_content,
                    embed: Some(embed),
                }).await?;

                // DB保存（トランザクション対応メソッド使用）
                battle_recruitment_repo.create_with_txn(
                    tx_ctx.sea_orm_txn(),
                    guild_id as i64,
                    channel_id as i64,
                    discord_result.message_id as i64,
                    quest.target_id,
                    battle_type as i32,
                    expiry_date,
                ).await?;

                Ok(discord_result.message_id)
            })
        }).await
    }
}
```

### 5.2 Repository層でのステートレス設計

**設計日時**: 2025年12月3日（リファクタリング実施）

#### 設計原則

Repository層は以下の原則に従ってステートレス化されています：

1. **ステートレス構造体**: Repository構造体はフィールドを持たず、メソッドのみを提供
2. **外部依存性注入**: DB接続/トランザクションをメソッドパラメータとして受け取る
3. **静的ディスパッチ**: ジェネリック型パラメータによる型安全性とパフォーマンス
4. **柔軟な接続受け入れ**: `DatabaseConnection`と`DatabaseTransaction`の両方に対応

#### ステートレスリポジトリの実装例

```rust
/// ステートレスなリポジトリ構造体（フィールドなし）
pub struct BattleRecruitmentRepositoryImpl;

impl BattleRecruitmentRepositoryImpl {
    /// ステートレスなコンストラクタ（パラメータなし）
    pub fn new() -> Self {
        Self
    }

    /// ジェネリック型パラメータでDB接続を受け取るメソッド
    pub async fn get_by_message<'c, C>(
        &self,
        db: &'c C,  // DB接続またはトランザクション
        guild_id: u64,
        channel_id: u64,
        message_id: u64,
    ) -> Result<Option<BattleRecruitment>>
    where
        C: sea_orm::ConnectionTrait,  // 静的ディスパッチ
    {
        let result = battle_recruitments::Entity::find()
            .filter(battle_recruitments::Column::GuildId.eq(guild_id as i64))
            .filter(battle_recruitments::Column::ChannelId.eq(channel_id as i64))
            .filter(battle_recruitments::Column::MessageId.eq(message_id as i64))
            .one(db)  // ジェネリック型のDB接続を使用
            .await?;

        Ok(result.map(|model| model.into()))
    }

    /// トランザクション対応のcreateメソッド
    pub async fn create_with_txn(
        &self,
        txn: &DatabaseTransaction,  // トランザクション専用メソッド
        guild_id: i64,
        channel_id: i64,
        message_id: i64,
        target_id: i32,
        battle_type_id: i32,
        expiry_date: DateTime<Utc>,
    ) -> Result<BattleRecruitment> {
        let active_model = ActiveModel {
            guild_id: Set(guild_id),
            channel_id: Set(channel_id),
            message_id: Set(message_id),
            target_id: Set(target_id),
            battle_type_id: Set(battle_type_id),
            expiry_date: Set(expiry_date),
            ..Default::default()
        };

        let result = active_model
            .insert(txn)  // トランザクションを使用
            .await
            .map_err(|e| AppError::Database(e))?;

        Ok(BattleRecruitment::from(result))
    }
}
```

#### トレイトを使用したリポジトリの例

```rust
/// ステートレストレイト定義
#[async_trait]
pub trait GuildSpreadsheetConfigRepositoryTrait: Send + Sync {
    /// ジェネリック型パラメータでDB接続を受け取る
    async fn find_import_spreadsheet_id<'c, C>(
        &self,
        db: &'c C,
        guild_id: i64,
    ) -> Result<Option<String>, RepositoryError>
    where
        C: sea_orm::ConnectionTrait;

    /// トランザクション専用メソッド
    async fn upsert_import_spreadsheet_id(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        spreadsheet_id: &str,
    ) -> Result<(), RepositoryError>;
}

/// ステートレス実装
pub struct GuildSpreadsheetConfigRepository;

impl GuildSpreadsheetConfigRepository {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl GuildSpreadsheetConfigRepositoryTrait for GuildSpreadsheetConfigRepository {
    async fn find_import_spreadsheet_id<'c, C>(
        &self,
        db: &'c C,
        guild_id: i64,
    ) -> Result<Option<String>, RepositoryError>
    where
        C: sea_orm::ConnectionTrait,
    {
        let result = guild_spreadsheet_imports::Entity::find()
            .filter(guild_spreadsheet_imports::Column::GuildId.eq(guild_id))
            .one(db)  // ジェネリック型のDB接続を使用
            .await?;

        Ok(result.map(|model| model.spreadsheet_id))
    }

    async fn upsert_import_spreadsheet_id(
        &self,
        txn: &DatabaseTransaction,
        guild_id: i64,
        spreadsheet_id: &str,
    ) -> Result<(), RepositoryError> {
        let now = chrono::Utc::now();
        let active_model = guild_spreadsheet_imports::ActiveModel {
            guild_id: ActiveValue::Set(guild_id),
            spreadsheet_id: ActiveValue::Set(spreadsheet_id.to_string()),
            created_at: ActiveValue::Set(now),
            updated_at: ActiveValue::Set(now),
        };

        guild_spreadsheet_imports::Entity::insert(active_model)
            .on_conflict(/* ... */)
            .exec(txn)  // トランザクションを使用
            .await?;

        Ok(())
    }
}
```

#### ステートレス設計の利点

1. **テスタビリティの向上**: モック実装の注入が容易
2. **明示的な依存関係**: メソッドシグネチャから依存関係が明確
3. **柔軟性**: 任意のDB接続またはトランザクションを使用可能
4. **パフォーマンス**: 静的ディスパッチによるゼロコストの抽象化
5. **一貫性**: すべてのRepository層が同じパターンを使用

### 5.3 呼び出し側での使用パターン

#### Facade層での使用例

```rust
pub async fn register_guild_spreadsheets(
    &self,
    guild_id: i64,
    load_spreadsheet_id: &str,
    push_spreadsheet_id: &str,
) -> Result<RegistrationResult, FacadeError> {
    // トランザクション開始
    let txn = self.db.begin().await?;

    let result = async {
        // ステートレスなRepositoryを作成（パラメータなし）
        let repository = GuildSpreadsheetConfigRepository::new();

        // トランザクションをパラメータとして渡す
        repository
            .upsert_import_spreadsheet_id(&txn, guild_id, load_spreadsheet_id)
            .await?;

        repository
            .upsert_export_spreadsheet_id(&txn, guild_id, push_spreadsheet_id)
            .await?;

        Ok(())
    }
    .await;

    // エラーハンドリングとコミット/ロールバック
    match result {
        Ok(_) => {
            txn.commit().await?;
            Ok(/* ... */)
        }
        Err(e) => {
            txn.rollback().await?;
            Err(e)
        }
    }
}
```

#### Service層での使用例

```rust
pub struct NotificationHistoryService {
    db: DatabaseConnection,
    notification_repo: NotificationRepository,
}

impl NotificationHistoryService {
    pub fn new(db: DatabaseConnection) -> Self {
        // ステートレスなRepositoryを作成
        let notification_repo = NotificationRepository::new();
        Self { db, notification_repo }
    }

    pub async fn get_past_notifications(
        &self,
        guild_id: i64,
        days: i64,
    ) -> Result<Vec<notifications::Model>> {
        let now = Utc::now();
        let from = now - Duration::days(days);

        // DB接続を明示的に渡す
        let notifications = self
            .notification_repo
            .find_by_datetime_range(&self.db, from, now)
            .await?;

        Ok(notifications
            .into_iter()
            .filter(|n| n.guild_id == guild_id)
            .collect())
    }
}
```

#### Event層での使用例

```rust
pub async fn gspread_load(ctx: PoiseContext<'_>) -> Result<()> {
    let guild_id = ctx.guild_id().unwrap().get() as i64;
    let app_state = &ctx.data().app_state;

    // ステートレスなRepositoryを作成
    let repository = GuildSpreadsheetConfigRepository::new();

    // DB接続を明示的に渡す
    let spreadsheet_id = repository
        .find_import_spreadsheet_id(app_state.db(), guild_id)
        .await?;

    // 処理続行...
}
```

### 5.4 設計パターンのガイドライン

#### ✅ 推奨パターン

1. **ステートレスRepository**: Repositoryはフィールドを持たず、`new()`はパラメータなし
2. **外部依存性注入**: DB接続/トランザクションを常にメソッドパラメータとして渡す
3. **ジェネリック型パラメータ**: `<'c, C: ConnectionTrait>`を使用した静的ディスパッチ
4. **メソッド命名規則**:
   - トランザクション専用: `*_with_txn(txn: &DatabaseTransaction, ...)`
   - DB接続汎用: `*(db: &'c C, ...) where C: ConnectionTrait`
5. **明示的な依存関係**: メソッドシグネチャからDB接続の必要性が明確

#### ❌ 避けるべきパターン

1. **Repository内部でのDB接続保持**: `struct Repository { db: DatabaseConnection }` は禁止
2. **new()でのDB接続受け取り**: `Repository::new(db)` ではなく `Repository::new()`
3. **動的ディスパッチ**: `dyn ConnectionTrait` の使用は避ける（静的ディスパッチを優先）
4. **グローバル変数**: DB接続のグローバル変数化は厳禁
5. **Repository直接作成**: Facade層でのRepositoryContainer直接作成（TransactionManager推奨）

## 6. 拡張性と将来対応

### 6.1 他のORM対応

新しいORMを追加する場合の実装例：

#### Diesel対応

```rust
// 新しい実装を追加するだけ
pub struct DieselTransaction {
    inner: diesel::Connection,
}

impl DatabaseTransactionTrait for DieselTransaction {
    type Error = diesel::result::Error;

    async fn commit(self) -> Result<(), Self::Error> {
        // Diesel固有の実装
    }

    async fn rollback(self) -> Result<(), Self::Error> {
        // Diesel固有の実装
    }
}

pub struct DieselConnection {
    inner: diesel::PgConnection,
}

impl DatabaseConnectionTrait for DieselConnection {
    type Transaction = DieselTransaction;
    type Error = diesel::result::Error;

    async fn begin_transaction(&self) -> Result<Self::Transaction, Self::Error> {
        // Diesel固有のトランザクション開始
    }
}
```

### 6.2 テスタビリティ

モック実装の例：

```rust
pub struct MockTransaction;

impl DatabaseTransactionTrait for MockTransaction {
    type Error = std::io::Error;

    async fn commit(self) -> Result<(), Self::Error> {
        // テスト用の実装
        Ok(())
    }

    async fn rollback(self) -> Result<(), Self::Error> {
        // テスト用の実装
        Ok(())
    }
}
```

### 6.3 完全抽象化への移行

現在は実用性重視のSeaORM特化実装ですが、将来的には完全に型抽象化された版も実装可能：

```rust
pub struct TransactionManager<C: DatabaseConnectionTrait> {
    db_connection: Arc<C>,
    repos: RepositoryContainer,
}

impl<C: DatabaseConnectionTrait> TransactionManager<C>
where
    C::Error: Into<AppError>,
{
    // 完全に抽象化された実装
}
```

## 7. パフォーマンス考慮事項

### 7.1 接続管理

- **単一接続**: AppStateで単一のDB接続を共有
- **コネクションプール**: SeaORMの内蔵プールを活用
- **接続数最適化**: 不要な接続作成を回避

### 7.2 トランザクション管理

- **適切な境界**: Facade層でのトランザクション境界管理
- **長時間トランザクション回避**: 外部API呼び出しとの分離
- **リソース開放**: 自動的なcommit/rollback

### 7.3 メモリ効率

- **Arc使用**: 参照カウンタによる効率的な共有
- **不要なClone回避**: 借用とライフタイムの活用
- **ゼロコスト抽象化**: トレイト実装のオーバーヘッド最小化

## 8. エラーハンドリング戦略

### 8.1 エラー変換

- **統一エラー型**: すべてのエラーをAppErrorに変換
- **構造化エラー**: 明確なエラー分類とメッセージ
- **トレースログ**: 適切なログレベルでの問題追跡

### 8.2 トランザクションエラー

- **自動ロールバック**: エラー時の確実なロールバック
- **ログ出力**: トランザクションエラーの詳細記録
- **リトライ戦略**: 必要に応じたリトライメカニズム

## 9. 運用上の補足

本設計では、TransactionManager を中心とした依存関係の構成を明示しており、将来的な ORM 追加や抽象化の強化に対しても同じ責務分離を保つことを前提としています。互換性維持の判断はリリース計画側で行い、ここで定義した契約を満たす形で実装を更新してください。

## 10. リファクタリング履歴

### 10.1 Repository層のステートレス化（2025年12月3日）

**背景**: 従来のRepository層は内部にDB接続を保持するステートフルな設計でしたが、以下の問題がありました：
- テスト時のモック注入が困難
- 依存関係が暗黙的で不明確
- トランザクション管理との整合性が取りづらい

**実施内容**:
1. 全Repository構造体からDBフィールドを削除
2. `new()`メソッドをパラメータなしに変更
3. すべてのメソッドをジェネリック化（`<'c, C: ConnectionTrait>`）
4. 呼び出し箇所を更新してDB接続を明示的に渡すように修正

**対象Repository**（9件）:
- ChannelTypeRepository
- SeaOrmEnvironmentRepository
- SeaOrmBattleStyleRepository
- SeaOrmMessageTextRepository
- NotificationRepository
- ScheduleRepository
- LastProcessTimeRepository
- NotificationRelBattleRecruitmentRepository
- GuildSpreadsheetConfigRepository

**影響範囲**:
- Repository層: 9ファイル
- Facade層: 5ファイル（scheduler, cancel, change, new_recruit, guild_spreadsheet_registration_facade）
- Service層: 2ファイル（notification_service, guild_spreadsheet_config_service）
- Event層: 2ファイル（gspread_load, gspread_push）

**効果**:
- テスタビリティの向上（モック注入が容易）
- 明示的な依存関係（メソッドシグネチャから明確）
- 静的ディスパッチによるパフォーマンス向上
- コードベース全体での一貫性確保

## 11. 結論

現在の実装により以下の目標を達成：

1. **ORM非依存性**: 抽象化トレイトによる将来の拡張性
2. **アーキテクチャ改善**: クリーンアーキテクチャの原則遵守
3. **実用性**: 複雑すぎる抽象化を避けた実装
4. **保守性**: 明確な責務分離とエラーハンドリング
5. **ステートレス設計**: Repository層の外部依存性注入による柔軟性とテスタビリティの向上

この設計により、保守しやすく拡張可能なデータベース接続・トランザクション管理システムが実現されています。
