use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // 1. notificationsテーブルにtask_idカラムを追加（一時的にNULL許可）
        conn.execute_unprepared(
            "ALTER TABLE worker.notifications
             ADD COLUMN task_id INT",
        )
        .await?;

        // 2. 既存データを移行: scheduled_task_notificationsからtask_idをコピー
        conn.execute_unprepared(
            "UPDATE worker.notifications n
             SET task_id = stn.task_id
             FROM worker.scheduled_task_notifications stn
             WHERE n.id = stn.notification_id",
        )
        .await?;

        // 3. task_idにNOT NULL制約を追加
        conn.execute_unprepared(
            "ALTER TABLE worker.notifications
             ALTER COLUMN task_id SET NOT NULL",
        )
        .await?;

        // 4. 外部キー制約を追加（CASCADE削除）
        conn.execute_unprepared(
            "ALTER TABLE worker.notifications
             ADD CONSTRAINT fk_notifications_task_id
             FOREIGN KEY (task_id) REFERENCES worker.scheduled_tasks(id) ON DELETE CASCADE",
        )
        .await?;

        // 5. ユニーク制約を追加（1対1関係を保証）
        conn.execute_unprepared(
            "ALTER TABLE worker.notifications
             ADD CONSTRAINT uk_notifications_task_id UNIQUE (task_id)",
        )
        .await?;

        // 6. インデックスを作成
        conn.execute_unprepared(
            "CREATE INDEX idx_notifications_task_id ON worker.notifications(task_id)",
        )
        .await?;

        // 7. scheduled_task_notificationsテーブルを削除
        conn.execute_unprepared("DROP TABLE worker.scheduled_task_notifications")
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // 1. scheduled_task_notificationsテーブルを再作成
        conn.execute_unprepared(
            "CREATE TABLE worker.scheduled_task_notifications (
                task_id INT NOT NULL REFERENCES worker.scheduled_tasks(id) ON DELETE CASCADE,
                notification_id INT NOT NULL REFERENCES worker.notifications(id) ON DELETE CASCADE,
                PRIMARY KEY (task_id)
            )",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE INDEX idx_scheduled_task_notifications_notification
             ON worker.scheduled_task_notifications(notification_id)",
        )
        .await?;

        // 2. データを戻す
        conn.execute_unprepared(
            "INSERT INTO worker.scheduled_task_notifications (task_id, notification_id)
             SELECT task_id, id FROM worker.notifications
             WHERE task_id IS NOT NULL",
        )
        .await?;

        // 3. インデックスを削除
        conn.execute_unprepared("DROP INDEX IF EXISTS worker.idx_notifications_task_id")
            .await?;

        // 4. 制約を削除
        conn.execute_unprepared(
            "ALTER TABLE worker.notifications DROP CONSTRAINT IF EXISTS uk_notifications_task_id",
        )
        .await?;

        conn.execute_unprepared(
            "ALTER TABLE worker.notifications DROP CONSTRAINT IF EXISTS fk_notifications_task_id",
        )
        .await?;

        // 5. task_idカラムを削除
        conn.execute_unprepared("ALTER TABLE worker.notifications DROP COLUMN task_id")
            .await?;

        Ok(())
    }
}
