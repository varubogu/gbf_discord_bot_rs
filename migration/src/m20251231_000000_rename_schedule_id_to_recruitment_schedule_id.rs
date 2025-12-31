use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // scheduled_task_recurring_recruitments テーブルのカラムをリネーム
        // 1. task_id → scheduled_task_id
        conn.execute_unprepared(
            "ALTER TABLE worker.scheduled_task_recurring_recruitments
             RENAME COLUMN task_id TO scheduled_task_id",
        )
        .await?;

        // 2. schedule_id → recruitment_schedule_id
        conn.execute_unprepared(
            "ALTER TABLE worker.scheduled_task_recurring_recruitments
             RENAME COLUMN schedule_id TO recruitment_schedule_id",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // ロールバック（逆順で実行）
        // 1. recruitment_schedule_id → schedule_id
        conn.execute_unprepared(
            "ALTER TABLE worker.scheduled_task_recurring_recruitments
             RENAME COLUMN recruitment_schedule_id TO schedule_id",
        )
        .await?;

        // 2. scheduled_task_id → task_id
        conn.execute_unprepared(
            "ALTER TABLE worker.scheduled_task_recurring_recruitments
             RENAME COLUMN scheduled_task_id TO task_id",
        )
        .await?;

        Ok(())
    }
}
