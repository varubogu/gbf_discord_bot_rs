# パフォーマンスルール

## 基本方針

- **単一コネクションプール**: 最適化されたSeaORMコネクションプールを共有
- **メモリ効率**: 不要な文字列コピー、Cloneの削減
- **非同期最適化**: 適切なasync/await使用とブロッキング処理の回避
- **コンパイル時最適化**: 可能な限り`&'static str`や定数を使用
- **ゼロコスト抽象化**: 不要なArc、Cloneを避け、借用とライフタイムを活用

## DB接続管理

### コネクションプール最適化

```rust
// ✅ 推奨: 単一コネクションプールの共有
pub struct DatabaseConnectionManager {
    pool: DatabaseConnection,
}

impl DatabaseConnectionManager {
    pub async fn new() -> Result<Self, sea_orm::DbErr> {
        let pool = Database::connect(ConnectOptions::new(DATABASE_URL)
            .max_connections(20)  // 適切な接続数設定
            .min_connections(5)   // 最小接続数を設定
            .connect_timeout(Duration::from_secs(10))
            .idle_timeout(Duration::from_secs(600))
            .sqlx_logging_level(log::LevelFilter::Debug)
        ).await?;

        Ok(Self { pool })
    }

    pub fn get_connection(&self) -> &DatabaseConnection {
        &self.pool
    }
}

// ❌ 避けるべき: 複数のコネクションプール作成
impl SomeService {
    pub async fn new() -> Self {
        let pool = Database::connect(DATABASE_URL).await.unwrap(); // 避けるべき
        Self { pool }
    }
}
```

### 長時間トランザクションの回避

```rust
// ✅ 推奨: 短時間でのトランザクション実行
impl BattleRecruitmentFacade {
    pub async fn create_recruitment(&self, data: CreateRecruitmentData) -> Result<Recruitment> {
        self.tx_manager.execute_in_transaction(|tx| {
            Box::pin(async move {
                // 必要な処理のみをトランザクション内で実行
                let recruitment = self.recruitment_service.create(&data, &tx).await?;
                let message = self.message_service.prepare_message(&recruitment).await?;
                self.notification_service.save_pending(&message, &tx).await?;
                Ok(recruitment)
            })
        }).await
    }
}

// ❌ 避けるべき: 長時間トランザクション
impl BadFacade {
    pub async fn bad_create_recruitment(&self, data: CreateRecruitmentData) -> Result<Recruitment> {
        let tx = self.begin_transaction().await?;

        // 避けるべき: 外部API呼び出しをトランザクション内で実行
        let discord_message = self.discord_api.send_message().await?;

        // 避けるべき: 重い計算処理をトランザクション内で実行
        let complex_result = self.heavy_calculation().await?;

        let recruitment = self.save_recruitment(&data, &tx).await?;
        self.commit_transaction().await?;
        Ok(recruitment)
    }
}
```

### N+1問題の防止

```rust
// ✅ 推奨: 一括取得による最適化
impl BattleRecruitmentRepository {
    pub async fn find_with_participants(&self, ids: &[RecruitmentId], tx: &Transaction) -> Result<Vec<RecruitmentWithParticipants>> {
        // 一括でrecruitmentとparticipantsを取得
        let recruitments = recruitment::Entity::find()
            .filter(recruitment::Column::Id.is_in(ids.iter().map(|id| id.value())))
            .find_with_related(participant::Entity)
            .all(tx.as_ref())
            .await?;

        Ok(recruitments.into_iter().map(|(recruitment, participants)| {
            RecruitmentWithParticipants::new(recruitment.into(), participants.into_iter().map(Into::into).collect())
        }).collect())
    }
}

// ❌ 避けるべき: N+1クエリの発生
impl BadRepository {
    pub async fn bad_find_with_participants(&self, ids: &[RecruitmentId], tx: &Transaction) -> Result<Vec<RecruitmentWithParticipants>> {
        let mut results = Vec::new();

        for id in ids {
            // 避けるべき: ループ内での個別クエリ実行
            let recruitment = self.find_recruitment_by_id(id, tx).await?;
            let participants = self.find_participants_by_recruitment_id(id, tx).await?;
            results.push(RecruitmentWithParticipants::new(recruitment, participants));
        }

        Ok(results)
    }
}
```

## メモリ管理

### Arc<T>を用いた適切な参照共有

```rust
// ✅ 推奨: 必要な場合のみArcを使用
pub struct FacadeContainer {
    recruitment_facade: Arc<BattleRecruitmentFacade>,  // 複数箇所で共有される場合
    user_facade: UserFacade,  // 単一の所有者の場合はArc不要
}

impl FacadeContainer {
    pub fn new(
        recruitment_facade: Arc<BattleRecruitmentFacade>,
        user_facade: UserFacade,
    ) -> Self {
        Self { recruitment_facade, user_facade }
    }

    // ✅ 推奨: 必要な場合のみclone
    pub fn get_recruitment_facade(&self) -> Arc<BattleRecruitmentFacade> {
        Arc::clone(&self.recruitment_facade)  // 明示的なclone
    }
}

// ❌ 避けるべき: 不要なArcの使用
pub struct BadContainer {
    simple_value: Arc<String>,  // 避けるべき: 単純な値でのArc使用
    config: Arc<Config>,        // 避けるべき: 不変データでのArc使用
}
```

### 不要なclone()の回避

```rust
// ✅ 推奨: 借用の活用
impl MessageService {
    pub async fn format_recruitment_message(&self, recruitment: &Recruitment) -> String {
        format!(
            "募集: {} (ID: {})",
            recruitment.quest_name(),      // borrowingを活用
            recruitment.id().as_str()      // 参照を使用
        )
    }

    pub async fn process_multiple_recruitments(&self, recruitments: &[Recruitment]) -> Vec<String> {
        recruitments.iter()  // イテレータで借用
            .map(|recruitment| self.format_recruitment_message(recruitment))
            .collect()
    }
}

// ❌ 避けるべき: 不要なclone
impl BadMessageService {
    pub async fn bad_format_message(&self, recruitment: Recruitment) -> String {  // 避けるべき: 所有権を取る
        let cloned_recruitment = recruitment.clone();  // 避けるべき: 不要なclone
        format!("募集: {}", cloned_recruitment.quest_name())
    }
}
```

## 非同期最適化

### 適切なasync/await使用

```rust
// ✅ 推奨: 並行処理による最適化
impl RecruitmentNotificationService {
    pub async fn notify_all_participants(&self, recruitment_id: &RecruitmentId) -> Result<()> {
        let participants = self.participant_repo.find_by_recruitment_id(recruitment_id).await?;

        // 並行して通知を送信
        let notification_futures: Vec<_> = participants.iter()
            .map(|participant| self.send_notification(participant))
            .collect();

        futures::future::try_join_all(notification_futures).await?;
        Ok(())
    }

    async fn send_notification(&self, participant: &Participant) -> Result<()> {
        // 個別の通知処理
        self.discord_service.send_dm(participant.user_id(), "募集が更新されました").await
    }
}

// ❌ 避けるべき: 順次実行による性能劣化
impl BadNotificationService {
    pub async fn bad_notify_all(&self, recruitment_id: &RecruitmentId) -> Result<()> {
        let participants = self.participant_repo.find_by_recruitment_id(recruitment_id).await?;

        // 避けるべき: 順次実行
        for participant in participants {
            self.send_notification(&participant).await?;  // 1つずつ待機
        }

        Ok(())
    }
}
```

### ブロッキング処理の回避

```rust
use tokio::task;

// ✅ 推奨: CPU集約的処理の非同期化
impl RecruitmentAnalysisService {
    pub async fn analyze_recruitment_patterns(&self, data: AnalysisData) -> Result<AnalysisResult> {
        // CPU集約的な処理を別スレッドで実行
        let result = task::spawn_blocking(move || {
            // 重い計算処理
            self.perform_heavy_analysis(data)
        }).await??;

        Ok(result)
    }

    fn perform_heavy_analysis(&self, data: AnalysisData) -> Result<AnalysisResult> {
        // CPU集約的な処理の実装
        // ...
    }
}

// ❌ 避けるべき: async関数内でのブロッキング処理
impl BadAnalysisService {
    pub async fn bad_analyze(&self, data: AnalysisData) -> Result<AnalysisResult> {
        // 避けるべき: async関数内での重い同期処理
        let result = self.heavy_sync_calculation(data);  // これがasyncランタイムをブロック
        Ok(result)
    }
}
```

## コンパイル時最適化

### 静的文字列と定数の使用

```rust
// ✅ 推奨: 静的文字列の使用
pub const DEFAULT_QUEST_NAME: &str = "デフォルトクエスト";
pub const MAX_PARTICIPANTS: usize = 30;

pub struct MessageTemplates;

impl MessageTemplates {
    pub const RECRUITMENT_CREATED: &'static str = "募集を作成しました: {}";
    pub const RECRUITMENT_FULL: &'static str = "募集が満員になりました";
    pub const PARTICIPATION_SUCCESS: &'static str = "参加登録が完了しました";
}

impl MessageService {
    pub fn format_recruitment_created(&self, quest_name: &str) -> String {
        format!(MessageTemplates::RECRUITMENT_CREATED, quest_name)
    }
}

// ❌ 避けるべき: 動的文字列の多用
impl BadMessageService {
    pub fn bad_format_message(&self, message_type: &str, quest_name: &str) -> String {
        let template = match message_type {  // 避けるべき: 実行時の分岐
            "created" => "募集を作成しました: {}".to_string(),  // 避けるべき: 実行時の文字列生成
            "full" => "募集が満員になりました".to_string(),
            _ => "不明なメッセージ".to_string(),
        };
        format!(template, quest_name)
    }
}
```

### 効率的なコレクション操作

```rust
use std::collections::HashMap;

// ✅ 推奨: 効率的なコレクション操作
impl ParticipantManager {
    pub fn group_participants_by_role(&self, participants: Vec<Participant>) -> HashMap<Role, Vec<Participant>> {
        // 事前にキャパシティを指定
        let mut grouped = HashMap::with_capacity(4);  // 想定されるroleの数

        for participant in participants {
            grouped.entry(participant.role())
                .or_insert_with(Vec::new)  // 必要時のみVec作成
                .push(participant);
        }

        grouped
    }

    pub fn filter_active_participants(&self, participants: &[Participant]) -> Vec<&Participant> {
        participants.iter()
            .filter(|p| p.is_active())  // 借用のまま処理
            .collect()
    }
}

// ❌ 避けるべき: 非効率なコレクション操作
impl BadParticipantManager {
    pub fn bad_group_participants(&self, participants: Vec<Participant>) -> HashMap<Role, Vec<Participant>> {
        let mut grouped = HashMap::new();  // 避けるべき: キャパシティ未指定

        for participant in participants.clone() {  // 避けるべき: 不要なclone
            let role = participant.role().clone();  // 避けるべき: 不要なclone
            if !grouped.contains_key(&role) {       // 避けるべき: 2回のハッシュ計算
                grouped.insert(role.clone(), Vec::new());
            }
            grouped.get_mut(&role).unwrap().push(participant);
        }

        grouped
    }
}
```

## パフォーマンス計測とモニタリング

### メトリクス収集

```rust
use std::time::Instant;
use tracing::{info, warn};

// ✅ 推奨: パフォーマンス計測の実装
impl BattleRecruitmentFacade {
    pub async fn create_recruitment_with_metrics(&self, data: CreateRecruitmentData) -> Result<Recruitment> {
        let start_time = Instant::now();

        let result = self.create_recruitment(data).await;

        let duration = start_time.elapsed();
        info!(
            duration_ms = duration.as_millis(),
            operation = "create_recruitment",
            success = result.is_ok(),
            "募集作成処理完了"
        );

        if duration.as_millis() > 1000 {  // 1秒以上の場合は警告
            warn!(
                duration_ms = duration.as_millis(),
                "募集作成処理が遅延しています"
            );
        }

        result
    }
}
```

### メモリ使用量の監視

```rust
// ✅ 推奨: メモリ効率的な大量データ処理
impl RecruitmentBatchProcessor {
    pub async fn process_large_dataset(&self, recruitment_ids: Vec<RecruitmentId>) -> Result<()> {
        const BATCH_SIZE: usize = 100;

        // チャンクに分けて処理
        for chunk in recruitment_ids.chunks(BATCH_SIZE) {
            self.process_recruitment_chunk(chunk).await?;

            // GCを促すために明示的にdrop
            // 大量のデータを処理した後
        }

        Ok(())
    }

    async fn process_recruitment_chunk(&self, chunk: &[RecruitmentId]) -> Result<()> {
        let recruitments = self.recruitment_repo.find_by_ids(chunk).await?;

        for recruitment in recruitments {
            self.process_single_recruitment(recruitment).await?;
        }

        Ok(())
    }
}
```

## パフォーマンステストの指針

```rust
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_recruitment_creation_performance() {
        let facade = setup_test_facade().await;
        let start = Instant::now();

        // 100件の募集作成
        let mut tasks = Vec::new();
        for i in 0..100 {
            let facade_clone = facade.clone();
            tasks.push(tokio::spawn(async move {
                facade_clone.create_recruitment(create_test_data(i)).await
            }));
        }

        let results = futures::future::join_all(tasks).await;
        let duration = start.elapsed();

        // パフォーマンス要件の確認
        assert!(duration < Duration::from_secs(10), "100件の作成が10秒以内に完了すること");
        assert!(results.iter().all(|r| r.is_ok()), "すべての処理が成功すること");
    }

    #[tokio::test]
    async fn test_memory_usage_with_large_data() {
        let processor = setup_test_processor().await;

        // 大量データでのメモリ使用量テスト
        let large_dataset: Vec<RecruitmentId> = (0..10000)
            .map(|i| RecruitmentId::new(format!("test_{}", i)))
            .collect();

        let initial_memory = get_memory_usage();
        processor.process_large_dataset(large_dataset).await.unwrap();
        let final_memory = get_memory_usage();

        // メモリリークがないことを確認
        assert!(final_memory - initial_memory < 100 * 1024 * 1024, "メモリ使用量が100MB以下であること");
    }
}
```