use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // worker.battle_recruitments テーブルに host_discord_user_id カラムを追加
        conn.execute_unprepared(
            "ALTER TABLE worker.battle_recruitments
             ADD COLUMN host_discord_user_id BIGINT NOT NULL DEFAULT 0",
        )
        .await?;

        conn.execute_unprepared(
            "COMMENT ON COLUMN worker.battle_recruitments.host_discord_user_id
             IS '募集作成者（ホスト）のDiscordユーザーID。0は不明（マイグレーション前の旧データ）を表す。'",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // worker.battle_recruitments テーブルから host_discord_user_id カラムを削除
        conn.execute_unprepared(
            "ALTER TABLE worker.battle_recruitments
             DROP COLUMN host_discord_user_id",
        )
        .await?;

        Ok(())
    }
}
