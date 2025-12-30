use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // ========================================
        // 1. master.quests テーブルに sort_order 列を追加
        // ========================================
        conn.execute_unprepared(
            "ALTER TABLE master.quests
             ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0",
        )
        .await?;

        // ========================================
        // 2. guild_master.guild_quest_disables テーブルを作成
        // ========================================
        conn.execute_unprepared(
            "CREATE TABLE guild_master.guild_quest_disables (
                guild_id BIGINT NOT NULL,
                quest_id INTEGER NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (guild_id, quest_id)
            )",
        )
        .await?;

        // ========================================
        // 3. guild_master.guild_quest_disables の外部キー制約
        // ========================================
        // guild_master.guild_quest_disables → guild_master.guilds
        conn.execute_unprepared(
            "ALTER TABLE guild_master.guild_quest_disables
             ADD CONSTRAINT fk_guild_quest_disables_guild_id
             FOREIGN KEY (guild_id) REFERENCES guild_master.guilds(guild_id)
             ON DELETE CASCADE",
        )
        .await?;

        // guild_master.guild_quest_disables → master.quests
        conn.execute_unprepared(
            "ALTER TABLE guild_master.guild_quest_disables
             ADD CONSTRAINT fk_guild_quest_disables_quest_id
             FOREIGN KEY (quest_id) REFERENCES master.quests(id)
             ON DELETE CASCADE",
        )
        .await?;

        // ========================================
        // 4. guild_master.guild_quest_disables の RLS 設定
        // ========================================
        conn.execute_unprepared("ALTER TABLE guild_master.guild_quest_disables ENABLE ROW LEVEL SECURITY")
            .await?;

        conn.execute_unprepared(
            "CREATE POLICY guild_quest_disables_guild_policy ON guild_master.guild_quest_disables \
             FOR ALL TO gbf_bot_guild \
             USING (guild_id = current_setting('app.current_guild_id')::bigint)",
        )
        .await?;

        // ========================================
        // 5. guild_master.guild_quest_disables の権限設定
        // ========================================
        conn.execute_unprepared(
            "GRANT SELECT, INSERT, UPDATE, DELETE ON guild_master.guild_quest_disables
             TO gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        // ========================================
        // 6. インデックス作成
        // ========================================
        conn.execute_unprepared(
            "CREATE INDEX idx_guild_quest_disables_guild_id ON guild_master.guild_quest_disables(guild_id)",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE INDEX idx_guild_quest_disables_quest_id ON guild_master.guild_quest_disables(quest_id)",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // guild_master.guild_quest_disables テーブルを削除
        conn.execute_unprepared("DROP TABLE IF EXISTS guild_master.guild_quest_disables CASCADE")
            .await?;

        // master.quests の sort_order 列を削除
        conn.execute_unprepared("ALTER TABLE master.quests DROP COLUMN IF EXISTS sort_order")
            .await?;

        Ok(())
    }
}
