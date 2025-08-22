use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(BattleRecruitment::Table)
                    .if_not_exists()
                    .col(pk_auto(BattleRecruitment::Id))
                    .col(big_integer(BattleRecruitment::GuildId))
                    .col(big_integer(BattleRecruitment::ChannelId))
                    .col(big_integer(BattleRecruitment::MessageId))
                    .col(integer(BattleRecruitment::TargetId))
                    .col(integer(BattleRecruitment::BattleTypeId))
                    .col(timestamp_with_time_zone(BattleRecruitment::ExpiryDate))
                    .col(big_integer_null(BattleRecruitment::RecruitEndMessageId))
                    .col(timestamp_with_time_zone(BattleRecruitment::CreatedAt))
                    .col(timestamp_with_time_zone(BattleRecruitment::UpdatedAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(BattleRecruitment::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum BattleRecruitment {
    Table,
    Id,
    GuildId,
    ChannelId,
    MessageId,
    TargetId,
    BattleTypeId,
    ExpiryDate,
    RecruitEndMessageId,
    CreatedAt,
    UpdatedAt,
}
