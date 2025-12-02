use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // スキーマ作成
        conn.execute_unprepared("CREATE SCHEMA IF NOT EXISTS master")
            .await?;
        conn.execute_unprepared("CREATE SCHEMA IF NOT EXISTS guild_master")
            .await?;
        conn.execute_unprepared("CREATE SCHEMA IF NOT EXISTS worker")
            .await?;

        // スキーマへのUSAGE権限付与（全ロールに必要）
        // USAGE権限がないとスキーマ内のオブジェクトにアクセスできない
        conn.execute_unprepared(
            "GRANT USAGE ON SCHEMA master TO gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_admin"
        ).await?;

        conn.execute_unprepared(
            "GRANT USAGE ON SCHEMA guild_master TO gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_admin"
        ).await?;

        conn.execute_unprepared(
            "GRANT USAGE ON SCHEMA worker TO gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_admin"
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // スキーマ削除（CASCADE で全テーブル削除）
        conn.execute_unprepared("DROP SCHEMA IF EXISTS worker CASCADE")
            .await?;
        conn.execute_unprepared("DROP SCHEMA IF EXISTS guild_master CASCADE")
            .await?;
        conn.execute_unprepared("DROP SCHEMA IF EXISTS master CASCADE")
            .await?;

        Ok(())
    }
}
