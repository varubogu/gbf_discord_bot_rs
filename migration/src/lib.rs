pub use sea_orm_migration::prelude::*;

mod m20250822_233636_create_environments_table;
mod m20250822_233641_create_quests_table;
mod m20250822_233648_create_quests_alias_table;
mod m20250822_233654_create_message_texts_table;
mod m20250822_233700_create_battle_recruitments_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250822_233636_create_environments_table::Migration),
            Box::new(m20250822_233641_create_quests_table::Migration),
            Box::new(m20250822_233648_create_quests_alias_table::Migration),
            Box::new(m20250822_233654_create_message_texts_table::Migration),
            Box::new(m20250822_233700_create_battle_recruitments_table::Migration),
        ]
    }
}
