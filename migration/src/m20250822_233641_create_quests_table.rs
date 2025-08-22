use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Quest::Table)
                    .if_not_exists()
                    .col(pk_auto(Quest::Id))
                    .col(integer(Quest::TargetId))
                    .col(string(Quest::QuestName))
                    .col(integer(Quest::DefaultBattleType))
                    .col(timestamp_with_time_zone(Quest::CreatedAt))
                    .col(timestamp_with_time_zone(Quest::UpdatedAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Quest::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Quest {
    Table,
    Id,
    TargetId,
    QuestName,
    DefaultBattleType,
    CreatedAt,
    UpdatedAt,
}
