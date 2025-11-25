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
        ]
    }
}
