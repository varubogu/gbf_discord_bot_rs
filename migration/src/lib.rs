pub use sea_orm_migration::prelude::*;

mod m20250826_001129_init_schema;
mod m20250826_002000_add_timestamp_constraints;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250826_001129_init_schema::Migration),
            Box::new(m20250826_002000_add_timestamp_constraints::Migration),
        ]
    }
}
