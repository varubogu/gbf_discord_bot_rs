use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // battle_recruitment_dismissals テーブル作成
        conn.execute_unprepared(
            "CREATE TABLE worker.battle_recruitment_dismissals (
                id SERIAL PRIMARY KEY,
                recruitment_id INT NOT NULL REFERENCES worker.battle_recruitments(id) ON DELETE CASCADE,
                input_value TEXT NOT NULL,
                input_type INT NOT NULL,
                dismissal_datetime TIMESTAMPTZ,
                relative_days INT,
                relative_hours INT,
                relative_minutes INT,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE INDEX idx_battle_recruitment_dismissals_recruitment_id
            ON worker.battle_recruitment_dismissals(recruitment_id)",
        )
        .await?;

        // battle_recruitment_schedule_dismissals テーブル作成
        conn.execute_unprepared(
            "CREATE TABLE guild_master.battle_recruitment_schedule_dismissals (
                id SERIAL PRIMARY KEY,
                schedule_id INT NOT NULL REFERENCES guild_master.battle_recruitment_schedules(id) ON DELETE CASCADE,
                input_value TEXT NOT NULL,
                input_type INT NOT NULL,
                dismissal_time TIME,
                relative_days INT,
                relative_hours INT,
                relative_minutes INT,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE INDEX idx_battle_recruitment_schedule_dismissals_schedule_id
            ON guild_master.battle_recruitment_schedule_dismissals(schedule_id)",
        )
        .await?;

        // scheduled_task_dismissals テーブル作成
        conn.execute_unprepared(
            "CREATE TABLE worker.scheduled_task_dismissals (
                task_id INT PRIMARY KEY REFERENCES worker.scheduled_tasks(id) ON DELETE CASCADE,
                recruitment_dismissal_id INT NOT NULL REFERENCES worker.battle_recruitment_dismissals(id) ON DELETE CASCADE,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE INDEX idx_scheduled_task_dismissals_recruitment_dismissal_id
            ON worker.scheduled_task_dismissals(recruitment_dismissal_id)",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // 逆順で削除（外部キー制約を考慮）
        conn.execute_unprepared("DROP TABLE IF EXISTS worker.scheduled_task_dismissals")
            .await?;

        conn.execute_unprepared(
            "DROP TABLE IF EXISTS guild_master.battle_recruitment_schedule_dismissals",
        )
        .await?;

        conn.execute_unprepared("DROP TABLE IF EXISTS worker.battle_recruitment_dismissals")
            .await?;

        Ok(())
    }
}
