use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // 1. schedule_datetimeカラムに依存するインデックスを削除
        conn.execute_unprepared(
            "DROP INDEX IF EXISTS worker.idx_notifications_datetime",
        )
        .await?;

        // 2. schedule_datetimeカラムを削除
        conn.execute_unprepared(
            "ALTER TABLE worker.notifications
             DROP COLUMN IF EXISTS schedule_datetime",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // 1. schedule_datetimeカラムを再追加
        conn.execute_unprepared(
            "ALTER TABLE worker.notifications
             ADD COLUMN schedule_datetime TIMESTAMPTZ",
        )
        .await?;

        // 2. scheduled_tasksからschedule_datetimeをコピー
        conn.execute_unprepared(
            "UPDATE worker.notifications n
             SET schedule_datetime = st.schedule_datetime
             FROM worker.scheduled_tasks st
             WHERE n.task_id = st.id",
        )
        .await?;

        // 3. NOT NULL制約を追加
        conn.execute_unprepared(
            "ALTER TABLE worker.notifications
             ALTER COLUMN schedule_datetime SET NOT NULL",
        )
        .await?;

        // 4. インデックスを再作成
        conn.execute_unprepared(
            "CREATE INDEX idx_notifications_datetime
             ON worker.notifications(schedule_datetime)",
        )
        .await?;

        Ok(())
    }
}
