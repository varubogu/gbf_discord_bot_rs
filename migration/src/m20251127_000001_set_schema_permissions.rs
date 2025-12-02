use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // master スキーマの権限設定
        conn.execute_unprepared(
            "GRANT USAGE ON SCHEMA master TO gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        conn.execute_unprepared(
            "GRANT SELECT ON ALL TABLES IN SCHEMA master TO gbf_bot_system, gbf_bot_guild",
        )
        .await?;

        conn.execute_unprepared(
            "GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA master TO gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        // guild_master スキーマの権限設定
        conn.execute_unprepared(
            "GRANT USAGE ON SCHEMA guild_master TO gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        conn.execute_unprepared(
            "GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA guild_master TO gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        // worker スキーマの権限設定
        conn.execute_unprepared(
            "GRANT USAGE ON SCHEMA worker TO gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        conn.execute_unprepared(
            "GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA worker TO gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        // シーケンス権限設定
        // master スキーマ: system, guild は読み取り専用
        conn.execute_unprepared(
            "GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA master TO gbf_bot_system, gbf_bot_guild",
        )
        .await?;

        // master スキーマ: global, admin は INSERT 可能なので UPDATE 権限も必要
        conn.execute_unprepared(
            "GRANT USAGE, SELECT, UPDATE ON ALL SEQUENCES IN SCHEMA master TO gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        // guild_master スキーマ: 全ロールが INSERT 可能なので UPDATE 権限が必要
        conn.execute_unprepared(
            "GRANT USAGE, SELECT, UPDATE ON ALL SEQUENCES IN SCHEMA guild_master TO gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        // worker スキーマ: 全ロールが INSERT 可能なので UPDATE 権限が必要
        conn.execute_unprepared(
            "GRANT USAGE, SELECT, UPDATE ON ALL SEQUENCES IN SCHEMA worker TO gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // 権限を取り消し（ロールは削除しない）
        conn.execute_unprepared(
            "REVOKE ALL ON ALL SEQUENCES IN SCHEMA worker FROM gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        conn.execute_unprepared(
            "REVOKE ALL ON ALL SEQUENCES IN SCHEMA guild_master FROM gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        conn.execute_unprepared(
            "REVOKE ALL ON ALL SEQUENCES IN SCHEMA master FROM gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        conn.execute_unprepared(
            "REVOKE ALL ON ALL TABLES IN SCHEMA worker FROM gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        conn.execute_unprepared(
            "REVOKE ALL ON ALL TABLES IN SCHEMA guild_master FROM gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        conn.execute_unprepared(
            "REVOKE ALL ON ALL TABLES IN SCHEMA master FROM gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        conn.execute_unprepared(
            "REVOKE USAGE ON SCHEMA worker FROM gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        conn.execute_unprepared(
            "REVOKE USAGE ON SCHEMA guild_master FROM gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        conn.execute_unprepared(
            "REVOKE USAGE ON SCHEMA master FROM gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        Ok(())
    }
}
