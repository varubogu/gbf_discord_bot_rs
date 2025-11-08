use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // channel_types テーブルが存在しない場合は新規作成
        manager
            .create_table(
                Table::create()
                    .if_not_exists()
                    .table(ChannelTypes::Table)
                    .col(integer(ChannelTypes::Id).primary_key())
                    .col(string(ChannelTypes::Name).not_null())
                    .col(string_null(ChannelTypes::Memo))
                    .to_owned(),
            )
            .await?;

        // elements テーブルが存在しない場合は新規作成
        manager
            .create_table(
                Table::create()
                    .if_not_exists()
                    .table(Elements::Table)
                    .col(integer(Elements::Id).primary_key())
                    .col(string_null(Elements::ReactionStamp))
                    .col(string(Elements::NameJp).not_null())
                    .col(string_null(Elements::NameEn))
                    .to_owned(),
            )
            .await?;

        // battle_types テーブルが存在しない場合は新規作成
        manager
            .create_table(
                Table::create()
                    .if_not_exists()
                    .table(BattleTypes::Table)
                    .col(integer(BattleTypes::Id).primary_key())
                    .col(string(BattleTypes::DisplayName).not_null())
                    .col(string_null(BattleTypes::Reactions))
                    .col(integer(BattleTypes::SortOrder).not_null().default(0))
                    .col(timestamp_with_time_zone(BattleTypes::CreatedAt).not_null().default(Expr::current_timestamp()))
                    .col(timestamp_with_time_zone(BattleTypes::UpdatedAt).not_null().default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await?;

        // guilds テーブルが存在しない場合は新規作成
        manager
            .create_table(
                Table::create()
                    .if_not_exists()
                    .table(Guilds::Table)
                    .col(
                        ColumnDef::new(Guilds::GuildId)
                            .big_integer()
                            .not_null()
                            .primary_key()
                    )
                    .col(string(Guilds::Name).not_null())
                    .col(big_integer_null(Guilds::RecruitChannelId))
                    .col(string_null(Guilds::Timezone))
                    .col(integer_null(Guilds::DefaultRecruitDuration))
                    .col(timestamp_with_time_zone(Guilds::CreatedAt).not_null().default(Expr::current_timestamp()))
                    .col(timestamp_with_time_zone(Guilds::UpdatedAt).not_null().default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().if_exists().table(Guilds::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().if_exists().table(BattleTypes::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().if_exists().table(Elements::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().if_exists().table(ChannelTypes::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum ChannelTypes {
    Table,
    Id,
    Name,
    Memo,
}

#[derive(DeriveIden)]
enum Elements {
    Table,
    Id,
    ReactionStamp,
    NameJp,
    NameEn,
}

#[derive(DeriveIden)]
enum BattleTypes {
    Table,
    Id,
    DisplayName,
    Reactions,
    SortOrder,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Guilds {
    Table,
    GuildId,
    Name,
    RecruitChannelId,
    Timezone,
    DefaultRecruitDuration,
    CreatedAt,
    UpdatedAt,
}
