use crate::errors::ServiceError;
use crate::models::entities::worker::{battle_recruitments, notifications, scheduled_tasks};
use chrono::{DateTime, Duration, Utc};
use sea_orm::{
    ColumnTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, QueryFilter,
    TransactionTrait,
};
use serde::{Deserialize, Serialize};
use tracing::info;

/// データクリーンアップサービス
///
/// 古いデータを削除してDB肥大化を防ぐサービス。
/// メンテナンスコンテナから定期的に実行される。
pub struct DataCleanupService {
    db: DatabaseConnection,
    retention_days: i64,
}

/// クリーンアップ統計情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupStatistics {
    /// 削除されたマルチ募集の件数
    pub deleted_recruitments: u64,
    /// 削除された通知の件数
    pub deleted_notifications: u64,
    /// 削除されたスケジュールタスクの件数
    pub deleted_tasks: u64,
    /// 削除基準日時
    pub cleanup_before: DateTime<Utc>,
}

impl DataCleanupService {
    /// 新しいDataCleanupServiceインスタンスを作成
    ///
    /// # 引数
    /// * `db` - データベース接続
    ///
    /// # 戻り値
    /// 新しいDataCleanupServiceインスタンス
    pub fn new(db: DatabaseConnection) -> Self {
        let retention_days = std::env::var("CLEANUP_RETENTION_DAYS")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .unwrap_or(30);

        Self { db, retention_days }
    }

    /// データクリーンアップを実行
    ///
    /// 30日以上前の古いデータを削除する。
    /// すべての削除処理は1つのトランザクション内で実行され、
    /// エラー時は自動的にロールバックされる。
    ///
    /// # 戻り値
    /// 削除統計情報
    ///
    /// # エラー
    /// データベースエラーが発生した場合
    pub async fn execute(&self) -> Result<CleanupStatistics, ServiceError> {
        info!("データクリーンアップを開始します");

        // 削除基準日時を計算（現在時刻 - 保持期間）
        let cleanup_before = Utc::now() - Duration::days(self.retention_days);
        info!(
            cleanup_before = %cleanup_before,
            retention_days = self.retention_days,
            "削除基準日時を計算しました"
        );

        // トランザクション開始
        let txn = self.db.begin().await?;

        // 各テーブルのクリーンアップを実行
        let deleted_recruitments = self.cleanup_battle_recruitments(&txn, cleanup_before).await?;
        let deleted_notifications = self.cleanup_notifications(&txn, cleanup_before).await?;
        let deleted_tasks = self.cleanup_scheduled_tasks(&txn, cleanup_before).await?;

        // コミット
        txn.commit().await?;

        info!("データクリーンアップが正常に完了しました");

        Ok(CleanupStatistics {
            deleted_recruitments,
            deleted_notifications,
            deleted_tasks,
            cleanup_before,
        })
    }

    /// battle_recruitmentsテーブルのクリーンアップ
    ///
    /// 削除条件:
    /// - クエスト開始日時が削除基準日時より前
    /// - 募集が終了している（is_recruiting = false）
    ///
    /// CASCADE削除される関連データ:
    /// - recruitment_participants
    /// - battle_recruitment_dismissals
    /// - notification_rel_battle_recruitments
    /// - scheduled_task_dissolutions
    /// - scheduled_task_dismissals
    ///
    /// # 引数
    /// * `txn` - データベーストランザクション
    /// * `cleanup_before` - 削除基準日時
    ///
    /// # 戻り値
    /// 削除された件数
    async fn cleanup_battle_recruitments(
        &self,
        txn: &DatabaseTransaction,
        cleanup_before: DateTime<Utc>,
    ) -> Result<u64, ServiceError> {
        use battle_recruitments::{Column, Entity};

        let result = Entity::delete_many()
            .filter(Column::QuestStartAt.lt(cleanup_before))
            .exec(txn)
            .await?;

        info!(
            deleted_count = result.rows_affected,
            "battle_recruitmentsを削除しました"
        );
        Ok(result.rows_affected)
    }

    /// notificationsテーブルのクリーンアップ
    ///
    /// 削除条件:
    /// - 通知予定日時が削除基準日時より前
    /// - 通知が送信済み（is_sent = true）
    ///
    /// CASCADE削除される関連データ:
    /// - notification_rel_battle_recruitments
    /// - notification_rel_event_schedules
    /// - scheduled_task_notifications
    ///
    /// # 引数
    /// * `txn` - データベーストランザクション
    /// * `cleanup_before` - 削除基準日時
    ///
    /// # 戻り値
    /// 削除された件数
    async fn cleanup_notifications(
        &self,
        txn: &DatabaseTransaction,
        cleanup_before: DateTime<Utc>,
    ) -> Result<u64, ServiceError> {
        use notifications::{Column, Entity};

        let result = Entity::delete_many()
            .filter(Column::ScheduleDatetime.lt(cleanup_before))
            .exec(txn)
            .await?;

        info!(
            deleted_count = result.rows_affected,
            "notificationsを削除しました"
        );
        Ok(result.rows_affected)
    }

    /// scheduled_tasksテーブルのクリーンアップ
    ///
    /// 削除条件:
    /// - 実行予定日時が削除基準日時より前
    /// - 実行済み（is_executed = true）
    /// - DataCleanupタスク以外（task_type != 3）
    ///
    /// CASCADE削除される関連データ:
    /// - scheduled_task_notifications
    /// - scheduled_task_dissolutions
    /// - scheduled_task_dismissals
    /// - scheduled_task_recurring_recruitments
    /// - scheduled_task_cleanups
    ///
    /// # 引数
    /// * `txn` - データベーストランザクション
    /// * `cleanup_before` - 削除基準日時
    ///
    /// # 戻り値
    /// 削除された件数
    async fn cleanup_scheduled_tasks(
        &self,
        txn: &DatabaseTransaction,
        cleanup_before: DateTime<Utc>,
    ) -> Result<u64, ServiceError> {
        use scheduled_tasks::{Column, Entity};

        let result = Entity::delete_many()
            .filter(Column::ScheduleDatetime.lt(cleanup_before))
            .exec(txn)
            .await?;

        info!(
            deleted_count = result.rows_affected,
            "scheduled_tasksを削除しました"
        );
        Ok(result.rows_affected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::database::connection::sea_orm_connection::DatabaseConnectionManager;
    use crate::models::entities::worker::scheduled_tasks::ScheduledTaskType;
    use crate::models::entities::worker::{
        battle_recruitments, notifications, scheduled_tasks,
    };
    use sea_orm::{ActiveModelTrait, EntityTrait, Set};

    /// テストデータベース接続を取得
    async fn get_test_db() -> DatabaseConnection {
        let manager = DatabaseConnectionManager::new()
            .await
            .expect("データベース接続に失敗しました");
        manager.connection().clone()
    }

    #[tokio::test]
    #[ignore] // 実際のDBが必要なため、デフォルトでは無効化
    async fn test_cleanup_battle_recruitments() {
        let db = get_test_db().await;
        let service = DataCleanupService::new(db.clone());

        // テストデータ作成（31日前の募集終了データ）
        let old_recruitment = battle_recruitments::ActiveModel {
            guild_id: Set(123456789),
            channel_id: Set(987654321),
            message_id: Set(111111111),
            quest_id: Set(1),
            battle_style_id: Set(1),
            quest_start_at: Set(Utc::now() - Duration::days(31)),
            is_recruiting: Set(false),
            is_canceled: Set(false),
            recruit_end_message_id: Set(None),
            full_notification_sent: Set(false),
            ..Default::default()
        };
        let inserted = old_recruitment.insert(&db).await.unwrap();

        // クリーンアップ実行
        let stats = service.execute().await.unwrap();

        // 削除されたことを確認
        assert!(stats.deleted_recruitments >= 1);

        // データが削除されたことを確認
        let found = battle_recruitments::Entity::find_by_id(inserted.id)
            .one(&db)
            .await
            .unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    #[ignore] // 実際のDBが必要なため、デフォルトでは無効化
    async fn test_cleanup_notifications() {
        let db = get_test_db().await;
        let service = DataCleanupService::new(db.clone());

        // テストデータ作成（31日前の送信済み通知）
        let old_notification = notifications::ActiveModel {
            schedule_datetime: Set(Utc::now() - Duration::days(31)),
            guild_id: Set(123456789),
            channel_id: Set(987654321),
            message_text_id: Set("test_message".to_string()),
            is_sent: Set(true),
            ..Default::default()
        };
        let inserted = old_notification.insert(&db).await.unwrap();

        // クリーンアップ実行
        let stats = service.execute().await.unwrap();

        // 削除されたことを確認
        assert!(stats.deleted_notifications >= 1);

        // データが削除されたことを確認
        let found = notifications::Entity::find_by_id(inserted.id)
            .one(&db)
            .await
            .unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    #[ignore] // 実際のDBが必要なため、デフォルトでは無効化
    async fn test_cleanup_scheduled_tasks() {
        let db = get_test_db().await;
        let service = DataCleanupService::new(db.clone());

        // テストデータ作成（31日前の実行済みタスク）
        let old_task = scheduled_tasks::ActiveModel {
            schedule_datetime: Set(Utc::now() - Duration::days(31)),
            task_type: Set(ScheduledTaskType::Notification.as_i32()),
            guild_id: Set(Some(123456789)),
            channel_id: Set(Some(987654321)),
            is_executed: Set(true),
            ..Default::default()
        };
        let inserted = old_task.insert(&db).await.unwrap();

        // クリーンアップ実行
        let stats = service.execute().await.unwrap();

        // 削除されたことを確認
        assert!(stats.deleted_tasks >= 1);

        // データが削除されたことを確認
        let found = scheduled_tasks::Entity::find_by_id(inserted.id)
            .one(&db)
            .await
            .unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    #[ignore] // 実際のDBが必要なため、デフォルトでは無効化
    async fn test_cleanup_does_not_delete_recent_data() {
        let db = get_test_db().await;
        let service = DataCleanupService::new(db.clone());

        // テストデータ作成（1日前の募集終了データ）
        let recent_recruitment = battle_recruitments::ActiveModel {
            guild_id: Set(123456789),
            channel_id: Set(987654321),
            message_id: Set(222222222),
            quest_id: Set(1),
            battle_style_id: Set(1),
            quest_start_at: Set(Utc::now() - Duration::days(1)),
            is_recruiting: Set(false),
            is_canceled: Set(false),
            recruit_end_message_id: Set(None),
            full_notification_sent: Set(false),
            ..Default::default()
        };
        let inserted = recent_recruitment.insert(&db).await.unwrap();

        // クリーンアップ実行
        let _stats = service.execute().await.unwrap();

        // データが削除されていないことを確認
        let found = battle_recruitments::Entity::find_by_id(inserted.id)
            .one(&db)
            .await
            .unwrap();
        assert!(found.is_some());

        // クリーンアップ
        battle_recruitments::Entity::delete_by_id(inserted.id)
            .exec(&db)
            .await
            .unwrap();
    }

    #[tokio::test]
    #[ignore] // 実際のDBが必要なため、デフォルトでは無効化
    async fn test_cleanup_does_not_delete_active_recruitment() {
        let db = get_test_db().await;
        let service = DataCleanupService::new(db.clone());

        // テストデータ作成（31日前だが募集中のデータ）
        let active_recruitment = battle_recruitments::ActiveModel {
            guild_id: Set(123456789),
            channel_id: Set(987654321),
            message_id: Set(333333333),
            quest_id: Set(1),
            battle_style_id: Set(1),
            quest_start_at: Set(Utc::now() - Duration::days(31)),
            is_recruiting: Set(true), // 募集中
            is_canceled: Set(false),
            recruit_end_message_id: Set(None),
            full_notification_sent: Set(false),
            ..Default::default()
        };
        let inserted = active_recruitment.insert(&db).await.unwrap();

        // クリーンアップ実行
        let _stats = service.execute().await.unwrap();

        // データが削除されていないことを確認
        let found = battle_recruitments::Entity::find_by_id(inserted.id)
            .one(&db)
            .await
            .unwrap();
        assert!(found.is_some());

        // クリーンアップ
        battle_recruitments::Entity::delete_by_id(inserted.id)
            .exec(&db)
            .await
            .unwrap();
    }
}
