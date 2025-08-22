use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(MessageText::Table)
                    .if_not_exists()
                    .col(pk_auto(MessageText::Id))
                    .col(big_integer(MessageText::GuildId))
                    .col(string(MessageText::MessageId))
                    .col(string(MessageText::MessageJp))
                    .col(string_null(MessageText::MessageEn))
                    .col(timestamp_with_time_zone(MessageText::CreatedAt))
                    .col(timestamp_with_time_zone(MessageText::UpdatedAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(MessageText::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum MessageText {
    Table,
    Id,
    GuildId,
    MessageId,
    MessageJp,
    MessageEn,
    CreatedAt,
    UpdatedAt,
}
