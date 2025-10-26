pub use sea_orm_migration::prelude::*;

mod m20250826_001129_init_schema;
mod m20250826_002000_add_timestamp_constraints;
mod m20250826_070513_table_rebuild;
mod m20250826_195815_add_event_schedule_tables;
mod m20250827_053000_notification_relation_tables;
mod m20251026_000000_create_guild_spreadsheet_tables;

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
        ]
    }
}
