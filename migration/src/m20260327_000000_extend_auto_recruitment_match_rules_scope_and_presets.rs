use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        conn.execute_unprepared(
            "ALTER TABLE guild_master.auto_recruitment_match_rules
             DROP CONSTRAINT IF EXISTS fk_auto_recruitment_match_rules_guild",
        )
        .await?;

        conn.execute_unprepared(
            "ALTER TABLE guild_master.auto_recruitment_match_rules
             DROP CONSTRAINT IF EXISTS chk_auto_recruitment_match_rules_scope_guild_id",
        )
        .await?;

        conn.execute_unprepared(
            "ALTER TABLE guild_master.auto_recruitment_match_rules
             ADD CONSTRAINT chk_auto_recruitment_match_rules_scope_guild_id
             CHECK (guild_id >= 0)",
        )
        .await?;

        conn.execute_unprepared(
            "ALTER TABLE guild_master.auto_recruitment_match_rules
             DROP CONSTRAINT IF EXISTS chk_auto_recruitment_match_rules_preset_type",
        )
        .await?;

        conn.execute_unprepared(
            "ALTER TABLE guild_master.auto_recruitment_match_rules
             ADD CONSTRAINT chk_auto_recruitment_match_rules_preset_type CHECK (
                preset_type IN (
                    'min_members_only',
                    'one_each_element',
                    'specific_element_n_plus_any',
                    'earth_two_light_two_plus_any',
                    'fixed_element_quota'
                )
             )",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        conn.execute_unprepared(
            "DELETE FROM guild_master.auto_recruitment_match_rules
             WHERE guild_id = 0",
        )
        .await?;

        conn.execute_unprepared(
            "ALTER TABLE guild_master.auto_recruitment_match_rules
             DROP CONSTRAINT IF EXISTS chk_auto_recruitment_match_rules_scope_guild_id",
        )
        .await?;

        conn.execute_unprepared(
            "ALTER TABLE guild_master.auto_recruitment_match_rules
             DROP CONSTRAINT IF EXISTS chk_auto_recruitment_match_rules_preset_type",
        )
        .await?;

        conn.execute_unprepared(
            "ALTER TABLE guild_master.auto_recruitment_match_rules
             ADD CONSTRAINT chk_auto_recruitment_match_rules_preset_type CHECK (
                preset_type IN (
                    'min_members_only',
                    'one_each_element',
                    'specific_element_n_plus_any',
                    'fixed_element_quota'
                )
             )",
        )
        .await?;

        conn.execute_unprepared(
            "ALTER TABLE guild_master.auto_recruitment_match_rules
             ADD CONSTRAINT fk_auto_recruitment_match_rules_guild FOREIGN KEY (guild_id)
                 REFERENCES guild_master.guilds(guild_id) ON DELETE CASCADE",
        )
        .await?;

        Ok(())
    }
}
