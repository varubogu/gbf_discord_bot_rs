use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{DatabaseBackend, Statement};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // 各テーブルのcreated_atとupdated_atにデフォルト値とNOT NULL制約を追加

        // environments
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "ALTER TABLE environments ALTER COLUMN created_at SET DEFAULT NOW()",
        ))
        .await?;
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "ALTER TABLE environments ALTER COLUMN updated_at SET DEFAULT NOW()",
        ))
        .await?;

        // quests
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "ALTER TABLE quests ALTER COLUMN created_at SET DEFAULT NOW()",
        ))
        .await?;
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "ALTER TABLE quests ALTER COLUMN updated_at SET DEFAULT NOW()",
        ))
        .await?;

        // quest_aliases
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "ALTER TABLE quest_aliases ALTER COLUMN created_at SET DEFAULT NOW()",
        ))
        .await?;
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "ALTER TABLE quest_aliases ALTER COLUMN updated_at SET DEFAULT NOW()",
        ))
        .await?;

        // message_texts
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "ALTER TABLE message_texts ALTER COLUMN created_at SET DEFAULT NOW()",
        ))
        .await?;
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "ALTER TABLE message_texts ALTER COLUMN updated_at SET DEFAULT NOW()",
        ))
        .await?;

        // battle_recruitments
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "ALTER TABLE battle_recruitments ALTER COLUMN created_at SET DEFAULT NOW()",
        ))
        .await?;
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "ALTER TABLE battle_recruitments ALTER COLUMN updated_at SET DEFAULT NOW()",
        ))
        .await?;

        // event_schedules
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "ALTER TABLE event_schedules ALTER COLUMN created_at SET DEFAULT NOW()",
        ))
        .await?;
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "ALTER TABLE event_schedules ALTER COLUMN updated_at SET DEFAULT NOW()",
        ))
        .await?;

        // event_schedule_details
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "ALTER TABLE event_schedule_details ALTER COLUMN created_at SET DEFAULT NOW()",
        ))
        .await?;
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "ALTER TABLE event_schedule_details ALTER COLUMN updated_at SET DEFAULT NOW()",
        ))
        .await?;

        // last_process_times
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "ALTER TABLE last_process_times ALTER COLUMN created_at SET DEFAULT NOW()",
        ))
        .await?;
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "ALTER TABLE last_process_times ALTER COLUMN updated_at SET DEFAULT NOW()",
        ))
        .await?;

        // notifications (旧schedules)
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "ALTER TABLE notifications ALTER COLUMN created_at SET DEFAULT NOW()",
        ))
        .await?;
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "ALTER TABLE notifications ALTER COLUMN updated_at SET DEFAULT NOW()",
        ))
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // デフォルト値を削除
        let tables = vec![
            "environments",
            "quests",
            "quest_aliases",
            "message_texts",
            "battle_recruitments",
            "event_schedules",
            "event_schedule_details",
            "last_process_times",
            "notifications",
        ];

        for table in tables {
            db.execute(Statement::from_string(
                DatabaseBackend::Postgres,
                format!("ALTER TABLE {} ALTER COLUMN created_at DROP DEFAULT", table),
            ))
            .await?;
            db.execute(Statement::from_string(
                DatabaseBackend::Postgres,
                format!("ALTER TABLE {} ALTER COLUMN updated_at DROP DEFAULT", table),
            ))
            .await?;
        }

        Ok(())
    }
}
