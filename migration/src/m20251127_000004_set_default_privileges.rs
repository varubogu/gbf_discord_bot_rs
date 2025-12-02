use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // ========================================
        // 既存テーブルへの権限付与
        // ========================================

        // master スキーマ: system, guild は SELECT のみ
        conn.execute_unprepared(
            "GRANT SELECT ON ALL TABLES IN SCHEMA master TO gbf_bot_system, gbf_bot_guild",
        )
        .await?;

        conn.execute_unprepared(
            "GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA master TO gbf_bot_system, gbf_bot_guild",
        )
        .await?;

        // master スキーマ: global, admin は全権限
        conn.execute_unprepared(
            "GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA master TO gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        conn.execute_unprepared(
            "GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA master TO gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        // guild_master スキーマ: 全ロールに全権限
        conn.execute_unprepared(
            "GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA guild_master TO gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        conn.execute_unprepared(
            "GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA guild_master TO gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        // worker スキーマ: 全ロールに全権限
        conn.execute_unprepared(
            "GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA worker TO gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        conn.execute_unprepared(
            "GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA worker TO gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        // ========================================
        // デフォルト権限設定（将来作成されるテーブル用）
        // ========================================

        // master スキーマのデフォルト権限設定
        // system, guild ロールは SELECT のみ
        conn.execute_unprepared(
            "ALTER DEFAULT PRIVILEGES IN SCHEMA master \
             GRANT SELECT ON TABLES TO gbf_bot_system, gbf_bot_guild",
        )
        .await?;

        // global, admin ロールは全権限
        conn.execute_unprepared(
            "ALTER DEFAULT PRIVILEGES IN SCHEMA master \
             GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        // master スキーマのシーケンス: system, guild は読み取り専用
        conn.execute_unprepared(
            "ALTER DEFAULT PRIVILEGES IN SCHEMA master \
             GRANT USAGE, SELECT ON SEQUENCES TO gbf_bot_system, gbf_bot_guild",
        )
        .await?;

        // master スキーマのシーケンス: global, admin は INSERT 可能なので UPDATE 権限も必要
        conn.execute_unprepared(
            "ALTER DEFAULT PRIVILEGES IN SCHEMA master \
             GRANT USAGE, SELECT, UPDATE ON SEQUENCES TO gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        // guild_master スキーマのデフォルト権限設定
        conn.execute_unprepared(
            "ALTER DEFAULT PRIVILEGES IN SCHEMA guild_master \
             GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        // guild_master スキーマのシーケンス: 全ロールが INSERT 可能なので UPDATE 権限が必要
        conn.execute_unprepared(
            "ALTER DEFAULT PRIVILEGES IN SCHEMA guild_master \
             GRANT USAGE, SELECT, UPDATE ON SEQUENCES TO gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        // worker スキーマのデフォルト権限設定
        conn.execute_unprepared(
            "ALTER DEFAULT PRIVILEGES IN SCHEMA worker \
             GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        // worker スキーマのシーケンス: 全ロールが INSERT 可能なので UPDATE 権限が必要
        conn.execute_unprepared(
            "ALTER DEFAULT PRIVILEGES IN SCHEMA worker \
             GRANT USAGE, SELECT, UPDATE ON SEQUENCES TO gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // デフォルト権限の取り消し
        // worker スキーマ
        conn.execute_unprepared(
            "ALTER DEFAULT PRIVILEGES IN SCHEMA worker \
             REVOKE ALL ON SEQUENCES FROM gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        conn.execute_unprepared(
            "ALTER DEFAULT PRIVILEGES IN SCHEMA worker \
             REVOKE ALL ON TABLES FROM gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        // guild_master スキーマ
        conn.execute_unprepared(
            "ALTER DEFAULT PRIVILEGES IN SCHEMA guild_master \
             REVOKE ALL ON SEQUENCES FROM gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        conn.execute_unprepared(
            "ALTER DEFAULT PRIVILEGES IN SCHEMA guild_master \
             REVOKE ALL ON TABLES FROM gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        // master スキーマ
        conn.execute_unprepared(
            "ALTER DEFAULT PRIVILEGES IN SCHEMA master \
             REVOKE ALL ON SEQUENCES FROM gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        conn.execute_unprepared(
            "ALTER DEFAULT PRIVILEGES IN SCHEMA master \
             REVOKE ALL ON TABLES FROM gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        Ok(())
    }
}
