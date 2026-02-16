use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // 実行状態ENUMを作成
        conn.execute_unprepared(
            r#"
            DO $$
            BEGIN
                IF NOT EXISTS (
                    SELECT 1
                    FROM pg_type t
                    JOIN pg_namespace n ON n.oid = t.typnamespace
                    WHERE t.typname = 'task_execution_status'
                      AND n.nspname = 'worker'
                ) THEN
                    CREATE TYPE worker.task_execution_status AS ENUM (
                        'pending',
                        'succeeded',
                        'succeeded_with_warning',
                        'failed'
                    );
                END IF;
            END
            $$;
            "#,
        )
        .await?;

        // execution_status列を追加（既存データは一旦pending）
        conn.execute_unprepared(
            "ALTER TABLE worker.scheduled_tasks
             ADD COLUMN execution_status worker.task_execution_status
             NOT NULL DEFAULT 'pending'",
        )
        .await?;

        // 既存のis_executedをexecution_statusへ変換
        conn.execute_unprepared(
            "UPDATE worker.scheduled_tasks
             SET execution_status = 'succeeded'::worker.task_execution_status
             WHERE is_executed = true",
        )
        .await?;

        // 既存の部分インデックスを置き換え
        conn.execute_unprepared(
            "DROP INDEX IF EXISTS worker.idx_scheduled_tasks_datetime_not_executed",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE INDEX idx_scheduled_tasks_datetime_pending
             ON worker.scheduled_tasks(schedule_datetime)
             WHERE execution_status = 'pending'",
        )
        .await?;

        // 旧カラムを削除
        conn.execute_unprepared(
            "ALTER TABLE worker.scheduled_tasks
             DROP COLUMN is_executed",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // 旧カラムを復元
        conn.execute_unprepared(
            "ALTER TABLE worker.scheduled_tasks
             ADD COLUMN is_executed BOOLEAN NOT NULL DEFAULT FALSE",
        )
        .await?;

        // execution_statusをis_executedへ逆変換
        conn.execute_unprepared(
            "UPDATE worker.scheduled_tasks
             SET is_executed = CASE
                 WHEN execution_status = 'pending' THEN FALSE
                 ELSE TRUE
             END",
        )
        .await?;

        // 新インデックスを削除して旧インデックスを復元
        conn.execute_unprepared(
            "DROP INDEX IF EXISTS worker.idx_scheduled_tasks_datetime_pending",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE INDEX idx_scheduled_tasks_datetime_not_executed
             ON worker.scheduled_tasks(schedule_datetime)
             WHERE is_executed = false",
        )
        .await?;

        // execution_status列を削除
        conn.execute_unprepared(
            "ALTER TABLE worker.scheduled_tasks
             DROP COLUMN execution_status",
        )
        .await?;

        // ENUM型を削除
        conn.execute_unprepared(
            "DROP TYPE IF EXISTS worker.task_execution_status",
        )
        .await?;

        Ok(())
    }
}
