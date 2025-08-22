use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Environment::Table)
                    .if_not_exists()
                    .col(pk_auto(Environment::Id))
                    .col(string(Environment::Key))
                    .col(string(Environment::Value))
                    .col(timestamp_with_time_zone(Environment::CreatedAt))
                    .col(timestamp_with_time_zone(Environment::UpdatedAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Environment::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Environment {
    Table,
    Id,
    Key,
    Value,
    CreatedAt,
    UpdatedAt,
}
