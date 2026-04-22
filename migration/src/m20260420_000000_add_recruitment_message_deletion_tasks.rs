use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // 募集投稿削除タスクの関連テーブルを作成
        conn.execute_unprepared(
            "CREATE TABLE worker.scheduled_task_recruitment_message_deletions (
                task_id INT PRIMARY KEY REFERENCES worker.scheduled_tasks(id) ON DELETE CASCADE,
                recruitment_id INT NOT NULL REFERENCES worker.battle_recruitments(id) ON DELETE CASCADE,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE INDEX idx_scheduled_task_recruitment_message_deletions_recruitment
             ON worker.scheduled_task_recruitment_message_deletions(recruitment_id)",
        )
        .await?;

        conn.execute_unprepared(
            "GRANT SELECT, INSERT, UPDATE, DELETE
             ON worker.scheduled_task_recruitment_message_deletions
             TO gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        conn.execute_unprepared(
            "DO $$
             BEGIN
                 IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'gbf_bot_cleanup') THEN
                     GRANT DELETE, SELECT
                     ON worker.scheduled_task_recruitment_message_deletions
                     TO gbf_bot_cleanup;
                 END IF;
             END
             $$;",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        conn.execute_unprepared(
            "DO $$
             BEGIN
                 IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'gbf_bot_cleanup') THEN
                     REVOKE DELETE, SELECT
                     ON worker.scheduled_task_recruitment_message_deletions
                     FROM gbf_bot_cleanup;
                 END IF;
             END
             $$;",
        )
        .await?;

        conn.execute_unprepared(
            "DROP TABLE IF EXISTS worker.scheduled_task_recruitment_message_deletions",
        )
        .await?;

        Ok(())
    }
}
