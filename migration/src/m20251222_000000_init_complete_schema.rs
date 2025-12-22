use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // ========================================
        // 1. トリガー関数作成
        // ========================================
        conn.execute_unprepared(
            "CREATE OR REPLACE FUNCTION update_updated_at_column()
             RETURNS TRIGGER AS $$
             BEGIN
                 NEW.updated_at = CURRENT_TIMESTAMP;
                 RETURN NEW;
             END;
             $$ LANGUAGE plpgsql",
        )
        .await?;

        // ========================================
        // 2. スキーマ作成
        // ========================================
        conn.execute_unprepared("CREATE SCHEMA IF NOT EXISTS master")
            .await?;
        conn.execute_unprepared("CREATE SCHEMA IF NOT EXISTS guild_master")
            .await?;
        conn.execute_unprepared("CREATE SCHEMA IF NOT EXISTS worker")
            .await?;

        // スキーマへのUSAGE権限付与
        conn.execute_unprepared(
            "GRANT USAGE ON SCHEMA master TO gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_admin",
        )
        .await?;
        conn.execute_unprepared(
            "GRANT USAGE ON SCHEMA guild_master TO gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_admin",
        )
        .await?;
        conn.execute_unprepared(
            "GRANT USAGE ON SCHEMA worker TO gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        // ========================================
        // 3. master スキーマのテーブル作成
        // ========================================

        // master.battle_styles
        conn.execute_unprepared(
            "CREATE TABLE master.battle_styles (
                id INTEGER PRIMARY KEY NOT NULL,
                display_name VARCHAR NOT NULL,
                reactions VARCHAR,
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .await?;

        // master.channel_types
        conn.execute_unprepared(
            "CREATE TABLE master.channel_types (
                id INTEGER PRIMARY KEY NOT NULL,
                name VARCHAR NOT NULL,
                memo VARCHAR
            )",
        )
        .await?;

        // master.elements
        conn.execute_unprepared(
            "CREATE TABLE master.elements (
                id INTEGER PRIMARY KEY NOT NULL,
                reaction_stamp VARCHAR,
                name_jp VARCHAR NOT NULL,
                name_en VARCHAR
            )",
        )
        .await?;

        // master.environments
        conn.execute_unprepared(
            "CREATE TABLE master.environments (
                key VARCHAR PRIMARY KEY NOT NULL,
                value VARCHAR NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
            )",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE TRIGGER update_environments_updated_at
             BEFORE UPDATE ON master.environments
             FOR EACH ROW EXECUTE FUNCTION update_updated_at_column()",
        )
        .await?;

        // master.event_schedule_details
        conn.execute_unprepared(
            "CREATE TABLE master.event_schedule_details (
                id UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
                profile VARCHAR NOT NULL,
                start_day_relative VARCHAR NOT NULL,
                time VARCHAR NOT NULL,
                schedule_name VARCHAR NOT NULL,
                message_text_id VARCHAR NOT NULL,
                notification_channel_type INTEGER NOT NULL,
                reactions VARCHAR NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
            )",
        )
        .await?;

        // master.event_schedules
        conn.execute_unprepared(
            "CREATE TABLE master.event_schedules (
                id UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
                event_type VARCHAR NOT NULL,
                event_count BIGINT NOT NULL,
                profile VARCHAR NOT NULL,
                weak_attribute INTEGER NOT NULL,
                start_at TIMESTAMP NOT NULL,
                end_at TIMESTAMP NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                CONSTRAINT unique_event_type_count UNIQUE (event_type, event_count)
            )",
        )
        .await?;

        // master.message_texts
        conn.execute_unprepared(
            "CREATE TABLE master.message_texts (
                id VARCHAR PRIMARY KEY NOT NULL,
                message_jp VARCHAR NOT NULL,
                message_en VARCHAR,
                created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
            )",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE TRIGGER update_message_texts_updated_at
             BEFORE UPDATE ON master.message_texts
             FOR EACH ROW EXECUTE FUNCTION update_updated_at_column()",
        )
        .await?;

        // master.quests
        conn.execute_unprepared(
            "CREATE TABLE master.quests (
                id SERIAL PRIMARY KEY,
                name VARCHAR NOT NULL,
                default_battle_style_id INTEGER NOT NULL,
                recruit_count INTEGER NOT NULL DEFAULT 0,
                available_battle_style_ids TEXT NOT NULL DEFAULT '0',
                created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
            )",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE TRIGGER update_quests_updated_at
             BEFORE UPDATE ON master.quests
             FOR EACH ROW EXECUTE FUNCTION update_updated_at_column()",
        )
        .await?;

        // master.quest_aliases
        conn.execute_unprepared(
            "CREATE TABLE master.quest_aliases (
                quest_id INTEGER NOT NULL,
                sequence_no INTEGER NOT NULL,
                alias VARCHAR NOT NULL,
                alias_kana_small TEXT NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                PRIMARY KEY (quest_id, sequence_no)
            )",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE TRIGGER update_quest_aliases_updated_at
             BEFORE UPDATE ON master.quest_aliases
             FOR EACH ROW EXECUTE FUNCTION update_updated_at_column()",
        )
        .await?;

        // ========================================
        // 4. guild_master スキーマのテーブル作成
        // ========================================

        // guild_master.guilds
        conn.execute_unprepared(
            "CREATE TABLE guild_master.guilds (
                guild_id BIGINT PRIMARY KEY NOT NULL,
                name VARCHAR NOT NULL,
                recruit_channel_id BIGINT,
                timezone VARCHAR,
                default_recruit_duration INTEGER,
                created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .await?;

        // guild_master.guild_channels
        conn.execute_unprepared(
            "CREATE TABLE guild_master.guild_channels (
                guild_id BIGINT NOT NULL,
                channel_type INTEGER NOT NULL,
                channel_id BIGINT NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (guild_id, channel_type)
            )",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE INDEX idx_guild_channels_guild_id ON guild_master.guild_channels(guild_id)",
        )
        .await?;

        // guild_master.guild_environments
        conn.execute_unprepared(
            "CREATE TABLE guild_master.guild_environments (
                guild_id BIGINT NOT NULL,
                key VARCHAR NOT NULL,
                value VARCHAR NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (guild_id, key)
            )",
        )
        .await?;

        // guild_master.guild_event_schedule_details
        conn.execute_unprepared(
            "CREATE TABLE guild_master.guild_event_schedule_details (
                guild_id BIGINT NOT NULL,
                id UUID NOT NULL,
                profile VARCHAR NOT NULL,
                start_day_relative VARCHAR NOT NULL,
                time VARCHAR NOT NULL,
                schedule_name VARCHAR NOT NULL,
                message_text_id VARCHAR NOT NULL,
                notification_channel_type INTEGER NOT NULL,
                reactions VARCHAR NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (guild_id, id)
            )",
        )
        .await?;

        // guild_master.guild_event_schedules
        conn.execute_unprepared(
            "CREATE TABLE guild_master.guild_event_schedules (
                guild_id BIGINT NOT NULL,
                id UUID NOT NULL,
                event_type VARCHAR NOT NULL,
                event_count BIGINT NOT NULL,
                profile VARCHAR NOT NULL,
                weak_attribute INTEGER NOT NULL,
                start_at TIMESTAMP NOT NULL,
                end_at TIMESTAMP NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (guild_id, id)
            )",
        )
        .await?;

        // guild_master.guild_message_texts
        conn.execute_unprepared(
            "CREATE TABLE guild_master.guild_message_texts (
                guild_id BIGINT NOT NULL,
                id VARCHAR NOT NULL,
                message_jp VARCHAR NOT NULL,
                message_en VARCHAR,
                created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (guild_id, id)
            )",
        )
        .await?;

        // guild_master.guild_spreadsheet_exports
        conn.execute_unprepared(
            "CREATE TABLE guild_master.guild_spreadsheet_exports (
                guild_id BIGINT PRIMARY KEY NOT NULL,
                spreadsheet_id VARCHAR NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .await?;

        // guild_master.guild_spreadsheet_imports
        conn.execute_unprepared(
            "CREATE TABLE guild_master.guild_spreadsheet_imports (
                guild_id BIGINT PRIMARY KEY NOT NULL,
                spreadsheet_id VARCHAR NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .await?;

        // guild_master.guild_timezones
        conn.execute_unprepared(
            "CREATE TABLE guild_master.guild_timezones (
                guild_id BIGINT PRIMARY KEY NOT NULL,
                timezone VARCHAR NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .await?;

        // guild_master.all_recruitment_notification_roles
        conn.execute_unprepared(
            "CREATE SEQUENCE guild_master.all_recruitment_notification_roles_seq_seq",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE TABLE guild_master.all_recruitment_notification_roles (
                guild_id BIGINT NOT NULL,
                seq INTEGER NOT NULL DEFAULT nextval('guild_master.all_recruitment_notification_roles_seq_seq'),
                role_id BIGINT NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (guild_id, seq)
            )",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE INDEX idx_all_recruitment_notification_roles_guild_id
             ON guild_master.all_recruitment_notification_roles(guild_id)",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE UNIQUE INDEX uq_all_recruitment_notification_roles_guild_role
             ON guild_master.all_recruitment_notification_roles(guild_id, role_id)",
        )
        .await?;

        // guild_master.quest_recruitment_notification_roles
        conn.execute_unprepared(
            "CREATE SEQUENCE guild_master.quest_recruitment_notification_roles_seq_seq",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE TABLE guild_master.quest_recruitment_notification_roles (
                guild_id BIGINT NOT NULL,
                quest_id INTEGER NOT NULL,
                seq INTEGER NOT NULL DEFAULT nextval('guild_master.quest_recruitment_notification_roles_seq_seq'),
                role_id BIGINT NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (guild_id, quest_id, seq)
            )",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE INDEX idx_quest_recruitment_notification_roles_guild_quest
             ON guild_master.quest_recruitment_notification_roles(guild_id, quest_id)",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE UNIQUE INDEX uq_quest_recruitment_notification_roles_guild_quest_role
             ON guild_master.quest_recruitment_notification_roles(guild_id, quest_id, role_id)",
        )
        .await?;

        // ========================================
        // 5. worker スキーマのテーブル作成
        // ========================================

        // worker.battle_recruitments
        conn.execute_unprepared(
            "CREATE TABLE worker.battle_recruitments (
                id SERIAL PRIMARY KEY,
                guild_id BIGINT NOT NULL,
                channel_id BIGINT NOT NULL,
                message_id BIGINT NOT NULL,
                quest_id INTEGER NOT NULL,
                battle_style_id INTEGER NOT NULL,
                quest_start_at TIMESTAMPTZ NOT NULL,
                is_recruiting BOOLEAN NOT NULL DEFAULT true,
                is_canceled BOOLEAN NOT NULL DEFAULT false,
                recruit_end_message_id BIGINT,
                created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
            )",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE TRIGGER update_battle_recruitments_updated_at
             BEFORE UPDATE ON worker.battle_recruitments
             FOR EACH ROW EXECUTE FUNCTION update_updated_at_column()",
        )
        .await?;

        // worker.notifications
        conn.execute_unprepared(
            "CREATE TABLE worker.notifications (
                id SERIAL PRIMARY KEY,
                schedule_datetime TIMESTAMPTZ NOT NULL,
                guild_id BIGINT NOT NULL,
                channel_id BIGINT NOT NULL,
                message_text_id VARCHAR NOT NULL,
                is_sent BOOLEAN NOT NULL DEFAULT false,
                created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
            )",
        )
        .await?;

        // worker.last_process_times
        conn.execute_unprepared(
            "CREATE TABLE worker.last_process_times (
                process_type INTEGER PRIMARY KEY NOT NULL,
                execute_time TIMESTAMPTZ,
                memo VARCHAR NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
            )",
        )
        .await?;

        // worker.guild_last_process_times
        conn.execute_unprepared(
            "CREATE TABLE worker.guild_last_process_times (
                guild_id BIGINT NOT NULL,
                process_type INTEGER NOT NULL,
                execute_time TIMESTAMPTZ,
                memo VARCHAR NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (guild_id, process_type)
            )",
        )
        .await?;

        // worker.recruitment_participants
        conn.execute_unprepared(
            "CREATE TABLE worker.recruitment_participants (
                id BIGSERIAL PRIMARY KEY,
                recruitment_id INTEGER NOT NULL,
                user_id BIGINT NOT NULL,
                element_id INTEGER,
                participated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE INDEX idx_recruitment_participants_recruitment_id
             ON worker.recruitment_participants(recruitment_id)",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE INDEX idx_recruitment_participants_user_id
             ON worker.recruitment_participants(user_id)",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE UNIQUE INDEX idx_recruitment_participants_unique
             ON worker.recruitment_participants(recruitment_id, user_id, element_id)",
        )
        .await?;

        // worker.battle_recruitment_schedules
        conn.execute_unprepared(
            "CREATE TABLE worker.battle_recruitment_schedules (
                id SERIAL PRIMARY KEY,
                name VARCHAR NOT NULL DEFAULT '未設定',
                guild_id BIGINT NOT NULL,
                channel_id BIGINT NOT NULL,
                quest_id INTEGER NOT NULL,
                battle_style_id INTEGER NOT NULL,
                quest_start_time TIME NOT NULL,
                recruit_start_day_offset INTEGER NOT NULL DEFAULT 0,
                recruit_start_time TIME,
                max_participants INTEGER,
                note TEXT,
                is_enabled BOOLEAN NOT NULL DEFAULT true,
                created_by BIGINT NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE INDEX idx_battle_recruitment_schedules_guild_enabled
             ON worker.battle_recruitment_schedules(guild_id, is_enabled)",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE INDEX idx_battle_recruitment_schedules_created_by
             ON worker.battle_recruitment_schedules(created_by)",
        )
        .await?;

        // worker.battle_recruitment_schedule_days
        conn.execute_unprepared(
            "CREATE TABLE worker.battle_recruitment_schedule_days (
                id SERIAL PRIMARY KEY,
                schedule_id INTEGER NOT NULL,
                day_of_week INTEGER NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE INDEX idx_battle_recruitment_schedule_days_schedule_id
             ON worker.battle_recruitment_schedule_days(schedule_id)",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE UNIQUE INDEX idx_battle_recruitment_schedule_days_unique
             ON worker.battle_recruitment_schedule_days(schedule_id, day_of_week)",
        )
        .await?;

        // worker.notification_rel_battle_recruitments
        conn.execute_unprepared(
            "CREATE TABLE worker.notification_rel_battle_recruitments (
                recruit_id INTEGER NOT NULL,
                notification_id INTEGER NOT NULL,
                created_at TIMESTAMPTZ NOT NULL,
                PRIMARY KEY (recruit_id, notification_id)
            )",
        )
        .await?;

        // worker.notification_rel_event_schedules
        conn.execute_unprepared(
            "CREATE TABLE worker.notification_rel_event_schedules (
                event_schedule_id UUID NOT NULL,
                notification_id INTEGER NOT NULL,
                event_schedule_detail_id UUID,
                created_at TIMESTAMPTZ NOT NULL,
                PRIMARY KEY (event_schedule_id, notification_id)
            )",
        )
        .await?;

        // ========================================
        // 6. 外部キー制約作成
        // ========================================

        // master.quest_aliases → master.quests
        conn.execute_unprepared(
            "ALTER TABLE master.quest_aliases
             ADD CONSTRAINT fk_quest_aliases_target_id
             FOREIGN KEY (quest_id) REFERENCES master.quests(id)
             ON UPDATE CASCADE ON DELETE CASCADE",
        )
        .await?;

        // guild_master.guild_channels → guild_master.guilds
        conn.execute_unprepared(
            "ALTER TABLE guild_master.guild_channels
             ADD CONSTRAINT fk_guild_channels_guild_id
             FOREIGN KEY (guild_id) REFERENCES guild_master.guilds(guild_id)
             ON DELETE CASCADE",
        )
        .await?;

        // guild_master.guild_channels → master.channel_types
        conn.execute_unprepared(
            "ALTER TABLE guild_master.guild_channels
             ADD CONSTRAINT fk_guild_channels_channel_type
             FOREIGN KEY (channel_type) REFERENCES master.channel_types(id)
             ON DELETE RESTRICT",
        )
        .await?;

        // guild_master.guild_environments → guild_master.guilds
        conn.execute_unprepared(
            "ALTER TABLE guild_master.guild_environments
             ADD CONSTRAINT fk_guild_environments_guild_id
             FOREIGN KEY (guild_id) REFERENCES guild_master.guilds(guild_id)
             ON DELETE CASCADE",
        )
        .await?;

        // guild_master.guild_event_schedule_details → guild_master.guilds
        conn.execute_unprepared(
            "ALTER TABLE guild_master.guild_event_schedule_details
             ADD CONSTRAINT fk_guild_event_schedule_details_guild_id
             FOREIGN KEY (guild_id) REFERENCES guild_master.guilds(guild_id)
             ON DELETE CASCADE",
        )
        .await?;

        // guild_master.guild_event_schedules → guild_master.guilds
        conn.execute_unprepared(
            "ALTER TABLE guild_master.guild_event_schedules
             ADD CONSTRAINT fk_guild_event_schedules_guild_id
             FOREIGN KEY (guild_id) REFERENCES guild_master.guilds(guild_id)
             ON DELETE CASCADE",
        )
        .await?;

        // guild_master.guild_message_texts → guild_master.guilds
        conn.execute_unprepared(
            "ALTER TABLE guild_master.guild_message_texts
             ADD CONSTRAINT fk_guild_message_texts_guild_id
             FOREIGN KEY (guild_id) REFERENCES guild_master.guilds(guild_id)
             ON DELETE CASCADE",
        )
        .await?;

        // guild_master.quest_recruitment_notification_roles → master.quests
        conn.execute_unprepared(
            "ALTER TABLE guild_master.quest_recruitment_notification_roles
             ADD CONSTRAINT fk_quest_recruitment_notification_roles_quest_id
             FOREIGN KEY (quest_id) REFERENCES master.quests(id)
             ON DELETE CASCADE",
        )
        .await?;

        // worker.guild_last_process_times → guild_master.guilds
        conn.execute_unprepared(
            "ALTER TABLE worker.guild_last_process_times
             ADD CONSTRAINT fk_guild_last_process_times_guild_id
             FOREIGN KEY (guild_id) REFERENCES guild_master.guilds(guild_id)
             ON DELETE CASCADE",
        )
        .await?;

        // worker.recruitment_participants → worker.battle_recruitments
        conn.execute_unprepared(
            "ALTER TABLE worker.recruitment_participants
             ADD CONSTRAINT fk_recruitment_participants_recruitment_id
             FOREIGN KEY (recruitment_id) REFERENCES worker.battle_recruitments(id)
             ON DELETE CASCADE",
        )
        .await?;

        // worker.recruitment_participants → master.elements
        conn.execute_unprepared(
            "ALTER TABLE worker.recruitment_participants
             ADD CONSTRAINT fk_recruitment_participants_element_id
             FOREIGN KEY (element_id) REFERENCES master.elements(id)
             ON DELETE RESTRICT",
        )
        .await?;

        // worker.battle_recruitment_schedules → guild_master.guilds
        conn.execute_unprepared(
            "ALTER TABLE worker.battle_recruitment_schedules
             ADD CONSTRAINT fk_battle_recruitment_schedules_guild_id
             FOREIGN KEY (guild_id) REFERENCES guild_master.guilds(guild_id)
             ON DELETE CASCADE",
        )
        .await?;

        // worker.battle_recruitment_schedules → master.quests
        conn.execute_unprepared(
            "ALTER TABLE worker.battle_recruitment_schedules
             ADD CONSTRAINT fk_battle_recruitment_schedules_quest_id
             FOREIGN KEY (quest_id) REFERENCES master.quests(id)
             ON DELETE RESTRICT",
        )
        .await?;

        // worker.battle_recruitment_schedules → master.battle_styles
        conn.execute_unprepared(
            "ALTER TABLE worker.battle_recruitment_schedules
             ADD CONSTRAINT fk_battle_recruitment_schedules_battle_style_id
             FOREIGN KEY (battle_style_id) REFERENCES master.battle_styles(id)
             ON DELETE RESTRICT",
        )
        .await?;

        // worker.battle_recruitment_schedule_days → worker.battle_recruitment_schedules
        conn.execute_unprepared(
            "ALTER TABLE worker.battle_recruitment_schedule_days
             ADD CONSTRAINT fk_battle_recruitment_schedule_days_schedule_id
             FOREIGN KEY (schedule_id) REFERENCES worker.battle_recruitment_schedules(id)
             ON DELETE CASCADE",
        )
        .await?;

        // worker.notification_rel_battle_recruitments → worker.notifications
        conn.execute_unprepared(
            "ALTER TABLE worker.notification_rel_battle_recruitments
             ADD CONSTRAINT fk_notification_rel_battle_recruitments_notification_id
             FOREIGN KEY (notification_id) REFERENCES worker.notifications(id)
             ON UPDATE CASCADE ON DELETE CASCADE",
        )
        .await?;

        // worker.notification_rel_battle_recruitments → worker.battle_recruitments
        conn.execute_unprepared(
            "ALTER TABLE worker.notification_rel_battle_recruitments
             ADD CONSTRAINT fk_notification_rel_battle_recruitments_recruit_id
             FOREIGN KEY (recruit_id) REFERENCES worker.battle_recruitments(id)
             ON UPDATE CASCADE ON DELETE CASCADE",
        )
        .await?;

        // worker.notification_rel_event_schedules → worker.notifications
        conn.execute_unprepared(
            "ALTER TABLE worker.notification_rel_event_schedules
             ADD CONSTRAINT fk_notification_rel_event_schedules_notification_id
             FOREIGN KEY (notification_id) REFERENCES worker.notifications(id)
             ON UPDATE CASCADE ON DELETE CASCADE",
        )
        .await?;

        // worker.notification_rel_event_schedules → master.event_schedules
        conn.execute_unprepared(
            "ALTER TABLE worker.notification_rel_event_schedules
             ADD CONSTRAINT fk_notification_rel_event_schedules_event_schedule_id
             FOREIGN KEY (event_schedule_id) REFERENCES master.event_schedules(id)
             ON UPDATE CASCADE ON DELETE CASCADE",
        )
        .await?;

        // worker.notification_rel_event_schedules → master.event_schedule_details
        conn.execute_unprepared(
            "ALTER TABLE worker.notification_rel_event_schedules
             ADD CONSTRAINT fk_notification_rel_event_schedules_event_schedule_detail_id
             FOREIGN KEY (event_schedule_detail_id) REFERENCES master.event_schedule_details(id)
             ON UPDATE CASCADE ON DELETE SET NULL",
        )
        .await?;

        // ========================================
        // 7. RLS (Row Level Security) 設定
        // ========================================

        // guild_master スキーマのテーブルに RLS を有効化
        let guild_master_tables = vec![
            "guilds",
            "guild_channels",
            "guild_spreadsheet_exports",
            "guild_spreadsheet_imports",
        ];

        for table in &guild_master_tables {
            conn.execute_unprepared(&format!(
                "ALTER TABLE guild_master.{} ENABLE ROW LEVEL SECURITY",
                table
            ))
            .await?;

            conn.execute_unprepared(&format!(
                "CREATE POLICY {}_guild_policy ON guild_master.{} \
                 FOR ALL TO gbf_bot_guild \
                 USING (guild_id = current_setting('app.current_guild_id')::bigint)",
                table, table
            ))
            .await?;
        }

        // worker スキーマのテーブルに RLS を有効化
        conn.execute_unprepared("ALTER TABLE worker.battle_recruitments ENABLE ROW LEVEL SECURITY")
            .await?;

        conn.execute_unprepared(
            "CREATE POLICY battle_recruitments_guild_policy ON worker.battle_recruitments \
             FOR ALL TO gbf_bot_guild \
             USING (guild_id = current_setting('app.current_guild_id')::bigint)",
        )
        .await?;

        conn.execute_unprepared("ALTER TABLE worker.notifications ENABLE ROW LEVEL SECURITY")
            .await?;

        conn.execute_unprepared(
            "CREATE POLICY notifications_guild_policy ON worker.notifications \
             FOR ALL TO gbf_bot_guild \
             USING (guild_id = current_setting('app.current_guild_id')::bigint) \
             WITH CHECK (guild_id = current_setting('app.current_guild_id')::bigint)",
        )
        .await?;

        conn.execute_unprepared(
            "ALTER TABLE worker.notification_rel_battle_recruitments ENABLE ROW LEVEL SECURITY",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE POLICY notification_rel_battle_recruitments_guild_policy ON worker.notification_rel_battle_recruitments \
             FOR ALL TO gbf_bot_guild \
             USING (EXISTS (\
                 SELECT 1 FROM worker.battle_recruitments br \
                 WHERE br.id = recruit_id \
                 AND br.guild_id = current_setting('app.current_guild_id')::bigint\
             ))",
        )
        .await?;

        conn.execute_unprepared(
            "ALTER TABLE worker.notification_rel_event_schedules ENABLE ROW LEVEL SECURITY",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE POLICY notification_rel_event_schedules_guild_policy ON worker.notification_rel_event_schedules \
             FOR ALL TO gbf_bot_guild \
             USING (true)",
        )
        .await?;

        conn.execute_unprepared("ALTER TABLE worker.last_process_times ENABLE ROW LEVEL SECURITY")
            .await?;

        conn.execute_unprepared(
            "CREATE POLICY last_process_times_guild_policy ON worker.last_process_times \
             FOR ALL TO gbf_bot_guild \
             USING (true)",
        )
        .await?;

        // ========================================
        // 8. テーブル権限設定
        // ========================================

        // master スキーマ: system, guild は SELECT のみ
        conn.execute_unprepared(
            "GRANT SELECT ON ALL TABLES IN SCHEMA master TO gbf_bot_system, gbf_bot_guild",
        )
        .await?;

        // master スキーマ: global, admin は全権限
        conn.execute_unprepared(
            "GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA master TO gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        // guild_master スキーマ: 全ロールに全権限
        conn.execute_unprepared(
            "GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA guild_master TO gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        // worker スキーマ: 全ロールに全権限
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

        // master スキーマ: global, admin は UPDATE 権限も必要
        conn.execute_unprepared(
            "GRANT USAGE, SELECT, UPDATE ON ALL SEQUENCES IN SCHEMA master TO gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        // guild_master スキーマ: 全ロールが UPDATE 権限が必要
        conn.execute_unprepared(
            "GRANT USAGE, SELECT, UPDATE ON ALL SEQUENCES IN SCHEMA guild_master TO gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        // worker スキーマ: 全ロールが UPDATE 権限が必要
        conn.execute_unprepared(
            "GRANT USAGE, SELECT, UPDATE ON ALL SEQUENCES IN SCHEMA worker TO gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        // ========================================
        // 9. デフォルト権限設定（将来作成されるテーブル用）
        // ========================================

        // master スキーマのデフォルト権限設定
        conn.execute_unprepared(
            "ALTER DEFAULT PRIVILEGES IN SCHEMA master \
             GRANT SELECT ON TABLES TO gbf_bot_system, gbf_bot_guild",
        )
        .await?;

        conn.execute_unprepared(
            "ALTER DEFAULT PRIVILEGES IN SCHEMA master \
             GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        conn.execute_unprepared(
            "ALTER DEFAULT PRIVILEGES IN SCHEMA master \
             GRANT USAGE, SELECT ON SEQUENCES TO gbf_bot_system, gbf_bot_guild",
        )
        .await?;

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

        conn.execute_unprepared(
            "ALTER DEFAULT PRIVILEGES IN SCHEMA worker \
             GRANT USAGE, SELECT, UPDATE ON SEQUENCES TO gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_admin",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // スキーマ削除（CASCADE で全テーブル、関数も削除）
        conn.execute_unprepared("DROP SCHEMA IF EXISTS worker CASCADE")
            .await?;
        conn.execute_unprepared("DROP SCHEMA IF EXISTS guild_master CASCADE")
            .await?;
        conn.execute_unprepared("DROP SCHEMA IF EXISTS master CASCADE")
            .await?;

        // トリガー関数削除
        conn.execute_unprepared("DROP FUNCTION IF EXISTS update_updated_at_column() CASCADE")
            .await?;

        Ok(())
    }
}
