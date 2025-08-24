# ログ・監視ルール

## 基本方針

- **ERROR**: システムエラー、予期しない例外
- **WARN**: 業務例外、リトライ可能なエラー
- **INFO**: 重要な業務処理の開始・終了
- **DEBUG**: 詳細なトレース情報
- **トランザクションIDによる処理追跡**: 一連の処理を追跡可能にする
- **メトリクス収集のための適切な情報出力**: パフォーマンス監視とメトリクス収集

## ログレベルの使い分け

### ERROR レベル

```rust
use tracing::error;

// ✅ 推奨: システムエラーでのERROR使用
impl DatabaseService {
    pub async fn connect(&self) -> Result<DatabaseConnection, DatabaseError> {
        match Database::connect(&self.url).await {
            Ok(conn) => Ok(conn),
            Err(err) => {
                error!(
                    error = %err,
                    database_url = %self.url,
                    "データベース接続に失敗しました"
                );
                Err(DatabaseError::ConnectionFailed { source: err })
            }
        }
    }
}
```

### WARN レベル

```rust
use tracing::warn;

// ✅ 推奨: 業務例外でのWARN使用
impl RecruitmentService {
    pub async fn add_participant(&self, recruitment_id: &RecruitmentId, participant: ParticipantData) -> Result<(), ServiceError> {
        match self.validate_recruitment_capacity(recruitment_id).await {
            Ok(_) => {
                // 正常処理
                self.repository.add_participant(recruitment_id, participant).await
            }
            Err(ServiceError::RecruitmentFull { id }) => {
                warn!(
                    recruitment_id = %id,
                    participant_user_id = %participant.user_id,
                    "募集が満員のため参加を拒否しました"
                );
                Err(ServiceError::RecruitmentFull { id })
            }
            Err(other_err) => Err(other_err),
        }
    }
}
```

### INFO レベル

```rust
use tracing::info;

// ✅ 推奨: 重要な業務処理でのINFO使用
impl BattleRecruitmentFacade {
    pub async fn create_recruitment(&self, data: CreateRecruitmentData) -> Result<Recruitment, FacadeError> {
        info!(
            quest_name = %data.quest_name,
            max_participants = data.max_participants,
            creator_id = %data.creator_id,
            "募集作成を開始しました"
        );

        let result = self.tx_manager.execute_in_transaction(|tx| {
            // 処理実行
        }).await;

        match &result {
            Ok(recruitment) => {
                info!(
                    recruitment_id = %recruitment.id(),
                    quest_name = %recruitment.quest_name(),
                    "募集作成が完了しました"
                );
            }
            Err(err) => {
                error!(
                    error = %err,
                    quest_name = %data.quest_name,
                    "募集作成に失敗しました"
                );
            }
        }

        result
    }
}
```

## 構造化ログの実装

### トランザクションIDによる処理追跡

```rust
use uuid::Uuid;
use tracing::{info, error, Span, instrument};

// ✅ 推奨: トランザクションIDを使用した処理追跡
#[derive(Clone, Debug)]
pub struct TransactionId(Uuid);

impl TransactionId {
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }
}

impl std::fmt::Display for TransactionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[instrument(skip(self), fields(tx_id = %tx_id))]
impl BattleRecruitmentFacade {
    pub async fn create_recruitment_with_tracking(
        &self,
        data: CreateRecruitmentData,
        tx_id: TransactionId
    ) -> Result<Recruitment, FacadeError> {
        info!("募集作成処理を開始");

        let result = self.quest_service.validate_quest(&data.quest_name, &tx_id).await
            .and_then(|_| self.recruitment_service.create(&data, &tx_id))
            .await;

        match &result {
            Ok(recruitment) => {
                info!(
                    recruitment_id = %recruitment.id(),
                    "募集作成処理が完了"
                );
            }
            Err(err) => {
                error!(error = %err, "募集作成処理でエラーが発生");
            }
        }

        result
    }
}
```

### メトリクス収集のための情報出力

```rust
use tracing::{info, debug};
use std::time::Instant;

// ✅ 推奨: メトリクス収集に適したログ出力
impl PerformanceLogger {
    pub fn log_operation_metrics<T>(
        operation_name: &str,
        result: &Result<T, impl std::error::Error>,
        duration: std::time::Duration,
        additional_fields: Option<&[(&str, &dyn std::fmt::Display)]>,
    ) {
        let mut fields = vec![
            ("operation", &operation_name as &dyn std::fmt::Display),
            ("duration_ms", &duration.as_millis()),
            ("success", &result.is_ok()),
        ];

        if let Some(extra) = additional_fields {
            fields.extend_from_slice(extra);
        }

        info!(
            operation = operation_name,
            duration_ms = duration.as_millis(),
            success = result.is_ok(),
            "操作メトリクス"
        );

        // パフォーマンス問題の検出
        if duration.as_millis() > 1000 {
            warn!(
                operation = operation_name,
                duration_ms = duration.as_millis(),
                "操作が1秒以上かかりました"
            );
        }
    }
}

// 使用例
impl BattleRecruitmentService {
    pub async fn create_with_metrics(&self, data: &CreateRecruitmentData) -> Result<Recruitment, ServiceError> {
        let start = Instant::now();
        let result = self.create_recruitment_internal(data).await;
        let duration = start.elapsed();

        PerformanceLogger::log_operation_metrics(
            "create_recruitment",
            &result,
            duration,
            Some(&[
                ("quest_name", &data.quest_name),
                ("max_participants", &data.max_participants),
            ])
        );

        result
    }
}
```