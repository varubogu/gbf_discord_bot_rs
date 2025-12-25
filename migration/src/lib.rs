pub use sea_orm_migration::prelude::*;

mod m20251222_000000_init_complete_schema;
mod m20251224_000000_rename_to_guild_settings_and_add_locale;
mod m20251225_000000_move_battle_recruitment_schedules_to_guild_master;
mod m20251225_010000_create_scheduled_tasks;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20251222_000000_init_complete_schema::Migration),
            Box::new(m20251224_000000_rename_to_guild_settings_and_add_locale::Migration),
            Box::new(m20251225_000000_move_battle_recruitment_schedules_to_guild_master::Migration),
            Box::new(m20251225_010000_create_scheduled_tasks::Migration),
        ]
    }
}
