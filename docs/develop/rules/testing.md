# テストルール

## 基本方針

- **各層での単体テスト必須**: すべての層で単体テストを実装する
- **モックオブジェクトによる依存関係の分離**: 外部依存関係はモックで分離する
- **テストダブルの適切な使用**: テストの目的に応じて適切なテストダブルを選択する
- **Facade層での結合テスト**: ビジネスロジックの統合テストをFacade層で実行
- **Repository層での実データベーステスト**: データアクセス層では実際のデータベースを使用したテストを実行（テスト用のユーザー・データベースを別途用意する）

## 単体テストの指針

### Service層のテスト

```rust
// ✅ 推奨: Service層の単体テスト
#[cfg(test)]
mod new_recruitment_service_tests {
    use super::*;
    use mockall::predicate::*;
    use tokio_test;

    // モックRepositoryの定義
    mockall::mock! {
        TestBattleRecruitmentRepository {}
        
        #[async_trait]
        impl BattleRecruitmentRepository for TestBattleRecruitmentRepository {
            async fn save(&self, recruitment: &BattleRecruitment, tx: &Transaction) -> Result<BattleRecruitment, DataAccessError>;
            async fn find_by_id(&self, id: &RecruitmentId, tx: &Transaction) -> Result<Option<BattleRecruitment>, DataAccessError>;
            async fn find_by_quest_name(&self, quest_name: &str, tx: &Transaction) -> Result<Vec<BattleRecruitment>, DataAccessError>;
        }
    }

    #[tokio::test]
    async fn test_create_recruitment_success() {
        // Arrange
        let mut mock_repo = MockTestBattleRecruitmentRepository::new();
        let expected_recruitment = create_test_recruitment("test_quest", 4);
        let expected_recruitment_clone = expected_recruitment.clone();

        mock_repo
            .expect_save()
            .with(eq(expected_recruitment.clone()), always())
            .times(1)
            .returning(move |_, _| Ok(expected_recruitment_clone.clone()));

        let service = NewRecruitmentService::new(Arc::new(mock_repo));
        let create_data = CreateRecruitmentData::new("test_quest", 4);
        let mock_tx = create_mock_transaction();

        // Act
        let result = service.create_recruitment(&create_data, &mock_tx).await;

        // Assert
        assert!(result.is_ok());
        let recruitment = result.unwrap();
        assert_eq!(recruitment.quest_name(), "test_quest");
        assert_eq!(recruitment.max_participants(), 4);
    }

    #[tokio::test]
    async fn test_create_recruitment_with_invalid_data() {
        // Arrange
        let mock_repo = MockTestBattleRecruitmentRepository::new();
        let service = NewRecruitmentService::new(Arc::new(mock_repo));
        let invalid_data = CreateRecruitmentData::new("", 0);  // 無効なデータ
        let mock_tx = create_mock_transaction();

        // Act
        let result = service.create_recruitment(&invalid_data, &mock_tx).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ServiceError::ValidationError { .. } => {} // 期待されるエラー
            _ => panic!("期待されるバリデーションエラーが発生しませんでした"),
        }
    }

    #[tokio::test]
    async fn test_create_recruitment_repository_error() {
        // Arrange
        let mut mock_repo = MockTestBattleRecruitmentRepository::new();
        mock_repo
            .expect_save()
            .returning(|_, _| Err(DataAccessError::ConnectionError {
                source: sea_orm::DbErr::ConnectionAcquire
            }));

        let service = NewRecruitmentService::new(Arc::new(mock_repo));
        let create_data = CreateRecruitmentData::new("test_quest", 4);
        let mock_tx = create_mock_transaction();

        // Act
        let result = service.create_recruitment(&create_data, &mock_tx).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ServiceError::DataAccess { .. } => {} // 期待されるエラー
            _ => panic!("期待されるデータアクセスエラーが発生しませんでした"),
        }
    }

    // テストヘルパー関数
    fn create_test_recruitment(quest_name: &str, max_participants: i32) -> BattleRecruitment {
        BattleRecruitment::new(
            RecruitmentId::generate(),
            UserId::new(123456789),
            quest_name.to_string(),
            max_participants,
            chrono::Utc::now(),
        )
    }

    fn create_mock_transaction() -> MockTransaction {
        // MockTransactionの作成
        MockTransaction::new()
    }
}
```

### Repository層のテスト（実データベース使用）

```rust
// ✅ 推奨: Repository層の実データベーステスト
#[cfg(test)]
mod battle_recruitment_repository_tests {
    use super::*;
    use sea_orm::*;
    use testcontainers::{clients::Cli, images::postgres::Postgres};

    struct TestContext {
        db: DatabaseConnection,
        _container: testcontainers::Container<'static, Postgres>,
    }

    impl TestContext {
        async fn new() -> Self {
            let docker = Cli::default();
            let container = docker.run(Postgres::default());
            let connection_string = format!(
                "postgres://postgres:postgres@127.0.0.1:{}/postgres",
                container.get_host_port_ipv4(5432)
            );

            let db = Database::connect(&connection_string).await.unwrap();

            // マイグレーション実行
            migration::Migrator::up(&db, None).await.unwrap();

            Self {
                db,
                _container: container,
            }
        }

        async fn begin_transaction(&self) -> DatabaseTransaction {
            self.db.begin().await.unwrap()
        }
    }

    #[tokio::test]
    async fn test_save_and_find_recruitment() {
        // Arrange
        let ctx = TestContext::new().await;
        let tx = ctx.begin_transaction().await;
        let repository = BattleRecruitmentRepository::new(ctx.db.clone());

        let recruitment = create_test_recruitment("test_quest_save", 6);

        // Act - 保存
        let saved_recruitment = repository.save(&recruitment, &tx).await.unwrap();

        // Act - 検索
        let found_recruitment = repository
            .find_by_id(saved_recruitment.id(), &tx)
            .await
            .unwrap()
            .unwrap();

        // Assert
        assert_eq!(found_recruitment.id(), saved_recruitment.id());
        assert_eq!(found_recruitment.quest_name(), "test_quest_save");
        assert_eq!(found_recruitment.max_participants(), 6);

        tx.rollback().await.unwrap();  // テスト後のクリーンアップ
    }

    #[tokio::test]
    async fn test_find_by_quest_name_multiple_results() {
        // Arrange
        let ctx = TestContext::new().await;
        let tx = ctx.begin_transaction().await;
        let repository = BattleRecruitmentRepository::new(ctx.db.clone());

        let recruitment1 = create_test_recruitment("same_quest", 4);
        let recruitment2 = create_test_recruitment("same_quest", 6);
        let recruitment3 = create_test_recruitment("different_quest", 8);

        repository.save(&recruitment1, &tx).await.unwrap();
        repository.save(&recruitment2, &tx).await.unwrap();
        repository.save(&recruitment3, &tx).await.unwrap();

        // Act
        let results = repository.find_by_quest_name("same_quest", &tx).await.unwrap();

        // Assert
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.quest_name() == "same_quest"));

        tx.rollback().await.unwrap();
    }

    #[tokio::test]
    async fn test_unique_constraint_violation() {
        // Arrange
        let ctx = TestContext::new().await;
        let tx = ctx.begin_transaction().await;
        let repository = BattleRecruitmentRepository::new(ctx.db.clone());

        let recruitment_id = RecruitmentId::generate();
        let recruitment1 = BattleRecruitment::with_id(
            recruitment_id.clone(),
            UserId::new(123),
            "test_quest".to_string(),
            4,
            chrono::Utc::now(),
        );
        let recruitment2 = BattleRecruitment::with_id(
            recruitment_id,  // 同じID
            UserId::new(456),
            "another_quest".to_string(),
            6,
            chrono::Utc::now(),
        );

        repository.save(&recruitment1, &tx).await.unwrap();

        // Act & Assert
        let result = repository.save(&recruitment2, &tx).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            DataAccessError::ConstraintViolation { .. } => {}, // 期待されるエラー
            _ => panic!("制約違反エラーが期待されましたが、異なるエラーが発生しました"),
        }

        tx.rollback().await.unwrap();
    }
}
```

## 結合テストの指針

### Facade層の結合テスト

```rust
// ✅ 推奨: Facade層での結合テスト
#[cfg(test)]
mod battle_recruitment_facade_integration_tests {
    use super::*;
    use std::sync::Arc;

    struct IntegrationTestContext {
        facade: BattleRecruitmentFacade,
        db: DatabaseConnection,
    }

    impl IntegrationTestContext {
        async fn new() -> Self {
            let db = setup_test_database().await;
            let tx_manager = setup_transaction_manager(db.clone()).await;
            let repositories = setup_repository_container(db.clone()).await;
            let services = setup_service_container(repositories).await;

            let facade = BattleRecruitmentFacade::new(
                tx_manager,
                services.new_recruitment_service,
                services.update_recruitment_service,
                services.participants_service,
            );

            Self { facade, db }
        }

        async fn cleanup(&self) {
            // テストデータのクリーンアップ
            let tx = self.db.begin().await.unwrap();
            // 必要に応じてテストデータを削除
            tx.rollback().await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_create_recruitment_end_to_end() {
        // Arrange
        let ctx = IntegrationTestContext::new().await;
        let create_data = CreateRecruitmentData::new("integration_test_quest", 8);

        // Act
        let result = ctx.facade.create_new_recruitment(create_data).await;

        // Assert
        assert!(result.is_ok());
        let recruitment_result = result.unwrap();
        assert!(recruitment_result.recruitment_id.is_valid());
        assert!(recruitment_result.message_id.is_some());

        // データベースに実際に保存されているかを確認
        let tx = ctx.db.begin().await.unwrap();
        let repo = BattleRecruitmentRepository::new(ctx.db.clone());
        let saved_recruitment = repo
            .find_by_id(&recruitment_result.recruitment_id, &tx)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(saved_recruitment.quest_name(), "integration_test_quest");
        assert_eq!(saved_recruitment.max_participants(), 8);

        tx.rollback().await.unwrap();
        ctx.cleanup().await;
    }

    #[tokio::test]
    async fn test_add_participant_to_recruitment() {
        // Arrange
        let ctx = IntegrationTestContext::new().await;
        let create_data = CreateRecruitmentData::new("participant_test_quest", 4);
        let recruitment_result = ctx.facade.create_new_recruitment(create_data).await.unwrap();

        let participant_data = ParticipantData::new(
            UserId::new(987654321),
            "attacker".to_string(),
        );

        // Act
        let result = ctx.facade.add_participant(
            &recruitment_result.recruitment_id,
            participant_data,
        ).await;

        // Assert
        assert!(result.is_ok());

        // 参加者が実際に追加されているかを確認
        let tx = ctx.db.begin().await.unwrap();
        let participants = ctx.facade.get_participants(&recruitment_result.recruitment_id, &tx).await.unwrap();
        assert_eq!(participants.len(), 1);
        assert_eq!(participants[0].user_id(), &UserId::new(987654321));
        assert_eq!(participants[0].role(), "attacker");

        tx.rollback().await.unwrap();
        ctx.cleanup().await;
    }

    #[tokio::test]
    async fn test_recruitment_full_scenario() {
        // Arrange
        let ctx = IntegrationTestContext::new().await;
        let create_data = CreateRecruitmentData::new("full_scenario_quest", 2);  // 最大2人
        let recruitment_result = ctx.facade.create_new_recruitment(create_data).await.unwrap();

        // Act & Assert - 1人目の参加者追加
        let participant1 = ParticipantData::new(UserId::new(111), "attacker".to_string());
        let result1 = ctx.facade.add_participant(&recruitment_result.recruitment_id, participant1).await;
        assert!(result1.is_ok());

        // Act & Assert - 2人目の参加者追加
        let participant2 = ParticipantData::new(UserId::new(222), "healer".to_string());
        let result2 = ctx.facade.add_participant(&recruitment_result.recruitment_id, participant2).await;
        assert!(result2.is_ok());

        // Act & Assert - 3人目の参加者追加（満員でエラーになるはず）
        let participant3 = ParticipantData::new(UserId::new(333), "support".to_string());
        let result3 = ctx.facade.add_participant(&recruitment_result.recruitment_id, participant3).await;
        assert!(result3.is_err());

        match result3.unwrap_err() {
            FacadeError::BusinessRule { source } => {
                assert!(source.to_string().contains("満員"));
            }
            _ => panic!("期待されるビジネスルールエラーが発生しませんでした"),
        }

        ctx.cleanup().await;
    }
}
```

## テストダブルの使い分け

### モック、スタブ、フェイクの適切な使用

```rust
// ✅ 推奨: テストダブルの適切な使い分け

// 1. Mock: 呼び出し回数や引数の検証が重要な場合
#[tokio::test]
async fn test_notification_service_calls_discord_api() {
    let mut mock_discord_service = MockDiscordService::new();
    mock_discord_service
        .expect_send_message()
        .with(eq("channel_123"), contains("募集を作成しました"))
        .times(1)  // 正確に1回呼び出されることを検証
        .returning(|_, _| Ok(MessageId::new(456)));

    let service = NotificationService::new(Arc::new(mock_discord_service));
    service.notify_recruitment_created("channel_123", "test_quest").await.unwrap();

    // モックが期待通りに呼び出されたかは、drop時に自動で検証される
}

// 2. Stub: 戻り値の制御が主目的の場合
#[tokio::test]
async fn test_quest_service_with_stubbed_repository() {
    let mut stub_repo = MockQuestRepository::new();
    stub_repo
        .expect_find_by_alias()
        .returning(|alias| {
            if alias == "known_quest" {
                Ok(Some(Quest::new("Known Quest", "raid")))
            } else {
                Ok(None)
            }
        });

    let service = QuestService::new(Arc::new(stub_repo));

    let known_quest = service.get_quest_info("known_quest").await.unwrap();
    assert!(known_quest.is_some());

    let unknown_quest = service.get_quest_info("unknown_quest").await.unwrap();
    assert!(unknown_quest.is_none());
}

// 3. Fake: より複雑な振る舞いが必要な場合
pub struct FakeUserRepository {
    users: std::sync::Mutex<std::collections::HashMap<UserId, User>>,
}

impl FakeUserRepository {
    pub fn new() -> Self {
        Self {
            users: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }

    pub fn add_user(&self, user: User) {
        self.users.lock().unwrap().insert(user.id().clone(), user);
    }
}

#[async_trait]
impl UserRepository for FakeUserRepository {
    async fn find_by_id(&self, id: &UserId, _tx: &Transaction) -> Result<Option<User>, DataAccessError> {
        Ok(self.users.lock().unwrap().get(id).cloned())
    }

    async fn save(&self, user: &User, _tx: &Transaction) -> Result<User, DataAccessError> {
        self.users.lock().unwrap().insert(user.id().clone(), user.clone());
        Ok(user.clone())
    }
}

#[tokio::test]
async fn test_user_service_with_fake_repository() {
    let fake_repo = Arc::new(FakeUserRepository::new());
    let initial_user = User::new(UserId::new(123), "test_user");
    fake_repo.add_user(initial_user.clone());

    let service = UserService::new(fake_repo.clone());
    let mock_tx = create_mock_transaction();

    // ユーザーの取得をテスト
    let found_user = service.find_user(&UserId::new(123), &mock_tx).await.unwrap();
    assert!(found_user.is_some());
    assert_eq!(found_user.unwrap().name(), "test_user");

    // 新しいユーザーの保存をテスト
    let new_user = User::new(UserId::new(456), "new_user");
    service.save_user(&new_user, &mock_tx).await.unwrap();

    // 保存されたユーザーが取得できることを確認
    let saved_user = service.find_user(&UserId::new(456), &mock_tx).await.unwrap();
    assert!(saved_user.is_some());
    assert_eq!(saved_user.unwrap().name(), "new_user");
}
```

## パフォーマンステスト

### 負荷テストの実装

```rust
// ✅ 推奨: パフォーマンステスト
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::{Duration, Instant};
    use tokio::task::JoinSet;

    #[tokio::test]
    async fn test_concurrent_recruitment_creation() {
        let ctx = IntegrationTestContext::new().await;
        let concurrent_count = 100;
        let mut join_set = JoinSet::new();

        let start_time = Instant::now();

        // 100個の募集を同時に作成
        for i in 0..concurrent_count {
            let facade = ctx.facade.clone();
            join_set.spawn(async move {
                let create_data = CreateRecruitmentData::new(
                    &format!("concurrent_quest_{}", i),
                    4,
                );
                facade.create_new_recruitment(create_data).await
            });
        }

        let mut success_count = 0;
        let mut error_count = 0;

        while let Some(result) = join_set.join_next().await {
            match result.unwrap() {
                Ok(_) => success_count += 1,
                Err(_) => error_count += 1,
            }
        }

        let duration = start_time.elapsed();

        // パフォーマンス要件の確認
        assert!(duration < Duration::from_secs(30), "100件の並行作成が30秒以内に完了すること");
        assert!(success_count > concurrent_count * 90 / 100, "成功率が90%以上であること");

        println!("並行処理結果: 成功 {}, 失敗 {}, 実行時間: {:?}",
                 success_count, error_count, duration);

        ctx.cleanup().await;
    }

    #[tokio::test]
    async fn test_large_participant_list_performance() {
        let ctx = IntegrationTestContext::new().await;
        let create_data = CreateRecruitmentData::new("large_list_quest", 1000);
        let recruitment_result = ctx.facade.create_new_recruitment(create_data).await.unwrap();

        let start_time = Instant::now();

        // 500人の参加者を追加
        for i in 0..500 {
            let participant_data = ParticipantData::new(
                UserId::new(1000 + i),
                "attacker".to_string(),
            );
            ctx.facade.add_participant(&recruitment_result.recruitment_id, participant_data)
                .await
                .unwrap();
        }

        let duration = start_time.elapsed();

        // パフォーマンス要件の確認
        assert!(duration < Duration::from_secs(60), "500人の参加者追加が60秒以内に完了すること");

        // 参加者リスト取得のパフォーマンステスト
        let fetch_start = Instant::now();
        let tx = ctx.db.begin().await.unwrap();
        let participants = ctx.facade.get_participants(&recruitment_result.recruitment_id, &tx).await.unwrap();
        let fetch_duration = fetch_start.elapsed();

        assert_eq!(participants.len(), 500);
        assert!(fetch_duration < Duration::from_millis(100), "500人の参加者リスト取得が100ms以内に完了すること");

        tx.rollback().await.unwrap();
        ctx.cleanup().await;
    }
}
```

## テスト環境の設定

### Docker Compose を使用したテスト環境

```yaml
# docker-compose.test.yml
version: '3.8'
services:
  test-db:
    image: postgres:15
    environment:
      POSTGRES_DB: test_gbf_bot
      POSTGRES_USER: test_user
      POSTGRES_PASSWORD: test_password
    ports:
      - "5433:5432"  # 本番DBと被らないポート
    volumes:
      - test_db_data:/var/lib/postgresql/data
    command: >
      postgres
      -c shared_preload_libraries=pg_stat_statements
      -c pg_stat_statements.max=10000
      -c pg_stat_statements.track=all

volumes:
  test_db_data:
```

### テスト用の環境設定

```rust
// tests/common/mod.rs
pub mod setup {
    use sea_orm::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    pub async fn setup_test_database() -> DatabaseConnection {
        INIT.call_once(|| {
            env_logger::init();
        });

        // 個別環境変数から接続URL構築
        let db_host = std::env::var("TEST_DB_HOST").unwrap_or_else(|_| "localhost".to_string());
        let db_port = std::env::var("TEST_DB_PORT").unwrap_or_else(|_| "5433".to_string());
        let db_name = std::env::var("TEST_DB_NAME").unwrap_or_else(|_| "test_gbf_bot".to_string());
        let db_user = std::env::var("TEST_DB_USER").unwrap_or_else(|_| "test_user".to_string());
        let db_password = std::env::var("TEST_DB_PASSWORD").unwrap_or_else(|_| "test_password".to_string());

        let database_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            db_user, db_password, db_host, db_port, db_name
        );

        let db = Database::connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        // テスト用のマイグレーション実行
        migration::Migrator::up(&db, None)
            .await
            .expect("Failed to run migrations");

        db
    }

    pub async fn cleanup_database(db: &DatabaseConnection) {
        // テストデータのクリーンアップ
        let tables = [
            "participants",
            "battle_recruitments",
            "users",
        ];

        for table in &tables {
            let _ = db.execute(Statement::from_string(
                DatabaseBackend::Postgres,
                format!("TRUNCATE TABLE {} RESTART IDENTITY CASCADE", table),
            )).await;
        }
    }
}
```

## CI/CDでのテスト実行

### GitHub Actions設定例

```yaml
# .github/workflows/test.yml
name: Tests

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main, develop ]

jobs:
  test:
    runs-on: ubuntu-latest

    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: postgres
          POSTGRES_DB: test_gbf_bot
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 5432:5432

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          profile: minimal
          override: true
          components: rustfmt, clippy

      - name: Cache dependencies
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Run unit tests
        run: cargo test --lib
        env:
          TEST_DB_HOST: localhost
          TEST_DB_PORT: 5432
          TEST_DB_NAME: test_gbf_bot
          TEST_DB_USER: postgres
          TEST_DB_PASSWORD: postgres

      - name: Run integration tests
        run: cargo test --test integration
        env:
          TEST_DB_HOST: localhost
          TEST_DB_PORT: 5432
          TEST_DB_NAME: test_gbf_bot
          TEST_DB_USER: postgres
          TEST_DB_PASSWORD: postgres

      - name: Run performance tests
        run: cargo test --test performance --release
        env:
          TEST_DB_HOST: localhost
          TEST_DB_PORT: 5432
          TEST_DB_NAME: test_gbf_bot
          TEST_DB_USER: postgres
          TEST_DB_PASSWORD: postgres

      - name: Check code formatting
        run: cargo fmt -- --check

      - name: Run clippy
        run: cargo clippy -- -D warnings

      - name: Generate coverage report
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --out xml

      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v3
```

## テストのベストプラクティス

### 1. テストの命名規則

```rust
// ✅ 推奨: 分かりやすいテスト名
#[tokio::test]
async fn test_create_recruitment_with_valid_data_returns_success() {
    // Given valid input data
    // When creating recruitment
    // Then should return success with recruitment ID
}

#[tokio::test]
async fn test_create_recruitment_with_empty_quest_name_returns_validation_error() {
    // Given empty quest name
    // When creating recruitment
    // Then should return validation error
}

// ❌ 避けるべき: 曖昧なテスト名
#[tokio::test]
async fn test_recruitment() {
    // 何をテストするか不明
}

#[tokio::test]
async fn test_error_case() {
    // どのようなエラーケースか不明
}
```

### 2. Given-When-Then パターン

```rust
#[tokio::test]
async fn test_add_participant_to_full_recruitment_returns_business_rule_error() {
    // Given: 満員の募集
    let ctx = TestContext::new().await;
    let recruitment = create_full_recruitment(&ctx).await;
    let new_participant = ParticipantData::new(UserId::new(999), "support".to_string());

    // When: 新しい参加者を追加
    let result = ctx.facade.add_participant(&recruitment.id(), new_participant).await;

    // Then: ビジネスルールエラーが返される
    assert!(result.is_err());
    match result.unwrap_err() {
        FacadeError::BusinessRule { source } => {
            assert!(source.to_string().contains("満員"));
        }
        _ => panic!("Expected BusinessRule error"),
    }
}
```

### 3. テストの独立性確保

```rust
// ✅ 推奨: 各テストが独立している
#[tokio::test]
async fn test_independent_test_1() {
    let ctx = TestContext::new().await;  // 独自のコンテキスト
    // テスト実行
    ctx.cleanup().await;  // 確実にクリーンアップ
}

#[tokio::test]
async fn test_independent_test_2() {
    let ctx = TestContext::new().await;  // 独自のコンテキスト
    // テスト実行
    ctx.cleanup().await;  // 確実にクリーンアップ
}

// ❌ 避けるべき: テスト間でデータを共有
static SHARED_DATA: Mutex<Vec<String>> = Mutex::new(Vec::new());

#[tokio::test]
async fn test_dependent_test_1() {
    SHARED_DATA.lock().unwrap().push("data1".to_string());
    // このテストは他のテストの実行順序に依存する
}
```