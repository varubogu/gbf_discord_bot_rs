use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // scheduled_tasks テーブル作成
        conn.execute_unprepared(
            "CREATE TABLE worker.scheduled_tasks (
                id SERIAL PRIMARY KEY,
                schedule_datetime TIMESTAMPTZ NOT NULL,
                task_type INT NOT NULL,
                guild_id BIGINT,
                channel_id BIGINT,
                is_executed BOOLEAN NOT NULL DEFAULT FALSE,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )",
        )
        .await?;

        // 部分インデックス: 未実行タスクに特化
        conn.execute_unprepared(
            "CREATE INDEX idx_scheduled_tasks_datetime_not_executed
            ON worker.scheduled_tasks(schedule_datetime)
            WHERE is_executed = false",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE INDEX idx_scheduled_tasks_type ON worker.scheduled_tasks(task_type)",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE INDEX idx_scheduled_tasks_guild ON worker.scheduled_tasks(guild_id)",
        )
        .await?;

        // scheduled_task_dissolutions テーブル作成
        conn.execute_unprepared(
            "CREATE TABLE worker.scheduled_task_dissolutions (
                task_id INT NOT NULL REFERENCES worker.scheduled_tasks(id) ON DELETE CASCADE,
                recruit_id INT NOT NULL REFERENCES worker.battle_recruitments(id) ON DELETE CASCADE,
                PRIMARY KEY (task_id)
            )",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE INDEX idx_scheduled_task_dissolutions_recruit
            ON worker.scheduled_task_dissolutions(recruit_id)",
        )
        .await?;

        // scheduled_task_notifications テーブル作成
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

        // scheduled_task_cleanups テーブル作成
        conn.execute_unprepared(
            "CREATE TABLE worker.scheduled_task_cleanups (
                task_id INT NOT NULL REFERENCES worker.scheduled_tasks(id) ON DELETE CASCADE,
                target_schema VARCHAR NOT NULL,
                target_table VARCHAR NOT NULL,
                cleanup_before TIMESTAMPTZ NOT NULL,
                PRIMARY KEY (task_id)
            )",
        )
        .await?;

        // scheduled_task_recurring_recruitments テーブル作成
        conn.execute_unprepared(
            "CREATE TABLE worker.scheduled_task_recurring_recruitments (
                task_id INT NOT NULL REFERENCES worker.scheduled_tasks(id) ON DELETE CASCADE,
                schedule_id INT NOT NULL REFERENCES guild_master.battle_recruitment_schedules(id) ON DELETE CASCADE,
                PRIMARY KEY (task_id)
            )",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE INDEX idx_scheduled_task_recurring_recruitments_schedule
            ON worker.scheduled_task_recurring_recruitments(schedule_id)",
        )
        .await?;

        // notifications テーブルに部分インデックスを追加（既存テーブルの最適化）
        conn.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_notifications_datetime_not_sent
            ON worker.notifications(schedule_datetime)
            WHERE is_sent = false",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // notifications の部分インデックスを削除
        conn.execute_unprepared(
            "DROP INDEX IF EXISTS worker.idx_notifications_datetime_not_sent",
        )
        .await?;

        // scheduled_task_recurring_recruitments テーブルを削除
        conn.execute_unprepared(
            "DROP TABLE IF EXISTS worker.scheduled_task_recurring_recruitments",
        )
        .await?;

        // scheduled_task_cleanups テーブルを削除
        conn.execute_unprepared(
            "DROP TABLE IF EXISTS worker.scheduled_task_cleanups",
        )
        .await?;

        // scheduled_task_notifications テーブルを削除
        conn.execute_unprepared(
            "DROP TABLE IF EXISTS worker.scheduled_task_notifications",
        )
        .await?;

        // scheduled_task_dissolutions テーブルを削除
        conn.execute_unprepared(
            "DROP TABLE IF EXISTS worker.scheduled_task_dissolutions",
        )
        .await?;

        // scheduled_tasks テーブルを削除
        conn.execute_unprepared(
            "DROP TABLE IF EXISTS worker.scheduled_tasks",
        )
        .await?;

        Ok(())
    }
}
