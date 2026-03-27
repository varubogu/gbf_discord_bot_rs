use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        conn.execute_unprepared(
            "CREATE TABLE guild_master.auto_recruitment_match_rules (
                guild_id BIGINT NOT NULL,
                quest_id INTEGER NOT NULL,
                preset_type VARCHAR(64) NOT NULL,
                min_match_count INTEGER NOT NULL,
                required_battle_style_id INTEGER,
                required_battle_style_count INTEGER,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                PRIMARY KEY (guild_id, quest_id),
                CONSTRAINT fk_auto_recruitment_match_rules_guild FOREIGN KEY (guild_id)
                    REFERENCES guild_master.guilds(guild_id) ON DELETE CASCADE,
                CONSTRAINT fk_auto_recruitment_match_rules_quest FOREIGN KEY (quest_id)
                    REFERENCES master.quests(id) ON DELETE CASCADE,
                CONSTRAINT fk_auto_recruitment_match_rules_style FOREIGN KEY (required_battle_style_id)
                    REFERENCES master.battle_styles(id) ON DELETE CASCADE,
                CONSTRAINT chk_auto_recruitment_match_rules_preset_type CHECK (
                    preset_type IN (
                        'min_members_only',
                        'one_each_element',
                        'specific_element_n_plus_any',
                        'fixed_element_quota'
                    )
                ),
                CONSTRAINT chk_auto_recruitment_match_rules_min_match_count CHECK (min_match_count >= 2),
                CONSTRAINT chk_auto_recruitment_match_rules_required_count CHECK (
                    required_battle_style_count IS NULL OR required_battle_style_count >= 1
                )
            )",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE TRIGGER set_auto_recruitment_match_rules_updated_at
            BEFORE UPDATE ON guild_master.auto_recruitment_match_rules
            FOR EACH ROW EXECUTE FUNCTION update_updated_at_column()",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE TABLE guild_master.auto_recruitment_match_rule_quotas (
                guild_id BIGINT NOT NULL,
                quest_id INTEGER NOT NULL,
                battle_style_id INTEGER NOT NULL,
                required_count INTEGER NOT NULL,
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                PRIMARY KEY (guild_id, quest_id, battle_style_id),
                CONSTRAINT fk_auto_recruitment_match_rule_quotas_rule FOREIGN KEY (guild_id, quest_id)
                    REFERENCES guild_master.auto_recruitment_match_rules(guild_id, quest_id) ON DELETE CASCADE,
                CONSTRAINT fk_auto_recruitment_match_rule_quotas_style FOREIGN KEY (battle_style_id)
                    REFERENCES master.battle_styles(id) ON DELETE CASCADE,
                CONSTRAINT chk_auto_recruitment_match_rule_quotas_required_count CHECK (required_count >= 1)
            )",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE TRIGGER set_auto_recruitment_match_rule_quotas_updated_at
            BEFORE UPDATE ON guild_master.auto_recruitment_match_rule_quotas
            FOR EACH ROW EXECUTE FUNCTION update_updated_at_column()",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        conn.execute_unprepared(
            "DROP TABLE IF EXISTS guild_master.auto_recruitment_match_rule_quotas",
        )
        .await?;

        conn.execute_unprepared("DROP TABLE IF EXISTS guild_master.auto_recruitment_match_rules")
            .await?;

        Ok(())
    }
}
