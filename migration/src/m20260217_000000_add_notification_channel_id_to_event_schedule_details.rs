use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // 通知先チャンネルID（直接指定）を追加
        conn.execute_unprepared(
            "ALTER TABLE guild_master.guild_event_schedule_details
             ADD COLUMN IF NOT EXISTS notification_channel_id BIGINT",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        conn.execute_unprepared(
            "ALTER TABLE guild_master.guild_event_schedule_details
             DROP COLUMN IF EXISTS notification_channel_id",
        )
        .await?;

        Ok(())
    }
}
