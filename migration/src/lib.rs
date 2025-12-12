pub use sea_orm_migration::prelude::*;

mod m20250826_001129_init_schema;
mod m20250826_002000_add_timestamp_constraints;
mod m20250826_070513_table_rebuild;
mod m20250826_195815_add_event_schedule_tables;
mod m20250827_053000_notification_relation_tables;
mod m20251026_000000_create_guild_spreadsheet_tables;
mod m20251102_000000_create_reference_master_tables;
mod m20251103_000001_update_channel_types;
mod m20251104_000001_align_schema_with_entities;
mod m20251107_000000_add_timestamp_defaults;
mod m20251107_000001_update_quest_aliases_primary_key;
mod m20251110_061759_rename_quest_aliases_id_to_sequence_no;
mod m20251121_000000_rename_battle_style_columns;
mod m20251121_000001_rename_battle_types_to_battle_styles;
mod m20251122_000000_add_recruitment_notification_message;
mod m20251124_create_guild_channels;
mod m20251126_000000_add_is_sent_to_notifications;
mod m20251127_000000_create_schemas;
mod m20251127_000001_set_schema_permissions;
mod m20251127_000002_move_tables_to_schemas;
mod m20251127_000003_enable_row_level_security;
mod m20251127_000004_set_default_privileges;
mod m20251203_000000_create_guild_override_tables;
mod m20251207_000000_create_recruitment_notification_roles;
mod m20251208_000000_create_guild_timezones;
mod m20251210_000000_create_recruitment_participants;
mod m20251211_000000_create_recruitment_schedules;
mod m20251212_000000_add_name_to_recruitment_schedules;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250826_001129_init_schema::Migration),
            Box::new(m20250826_002000_add_timestamp_constraints::Migration),
            Box::new(m20250826_195815_add_event_schedule_tables::Migration),
            Box::new(m20250826_070513_table_rebuild::Migration),
            Box::new(m20250827_053000_notification_relation_tables::Migration),
            Box::new(m20251026_000000_create_guild_spreadsheet_tables::Migration),
            Box::new(m20251102_000000_create_reference_master_tables::Migration),
            Box::new(m20251103_000001_update_channel_types::Migration),
            Box::new(m20251104_000001_align_schema_with_entities::Migration),
            Box::new(m20251107_000000_add_timestamp_defaults::Migration),
            Box::new(m20251107_000001_update_quest_aliases_primary_key::Migration),
            Box::new(m20251110_061759_rename_quest_aliases_id_to_sequence_no::Migration),
            Box::new(m20251121_000000_rename_battle_style_columns::Migration),
            Box::new(m20251121_000001_rename_battle_types_to_battle_styles::Migration),
            Box::new(m20251122_000000_add_recruitment_notification_message::Migration),
            Box::new(m20251124_create_guild_channels::Migration),
            Box::new(m20251126_000000_add_is_sent_to_notifications::Migration),
            Box::new(m20251127_000000_create_schemas::Migration),
            Box::new(m20251127_000001_set_schema_permissions::Migration),
            Box::new(m20251127_000002_move_tables_to_schemas::Migration),
            Box::new(m20251127_000003_enable_row_level_security::Migration),
            Box::new(m20251127_000004_set_default_privileges::Migration),
            Box::new(m20251203_000000_create_guild_override_tables::Migration),
            Box::new(m20251207_000000_create_recruitment_notification_roles::Migration),
            Box::new(m20251208_000000_create_guild_timezones::Migration),
            Box::new(m20251210_000000_create_recruitment_participants::Migration),
            Box::new(m20251211_000000_create_recruitment_schedules::Migration),
            Box::new(m20251212_000000_add_name_to_recruitment_schedules::Migration),
        ]
    }
}
