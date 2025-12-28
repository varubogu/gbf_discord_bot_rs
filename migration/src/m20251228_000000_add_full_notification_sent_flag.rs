use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // worker.battle_recruitments テーブルに full_notification_sent カラムを追加
        conn.execute_unprepared(
            "ALTER TABLE worker.battle_recruitments
             ADD COLUMN full_notification_sent BOOLEAN NOT NULL DEFAULT FALSE",
        )
        .await?;

        conn.execute_unprepared(
            "COMMENT ON COLUMN worker.battle_recruitments.full_notification_sent
             IS '規定人数到達時の通知が送信されたかどうか'",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // worker.battle_recruitments テーブルから full_notification_sent カラムを削除
        conn.execute_unprepared(
            "ALTER TABLE worker.battle_recruitments
             DROP COLUMN full_notification_sent",
        )
        .await?;

        Ok(())
    }
}
