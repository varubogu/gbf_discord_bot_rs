use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // guild_channelsテーブルを作成
        manager
            .create_table(
                Table::create()
                    .table(GuildChannels::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(GuildChannels::GuildId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GuildChannels::ChannelType)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GuildChannels::ChannelId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        timestamp_with_time_zone(GuildChannels::CreatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(GuildChannels::UpdatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .primary_key(
                        Index::create()
                            .col(GuildChannels::GuildId)
                            .col(GuildChannels::ChannelType),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_guild_channels_guild_id")
                            .from(GuildChannels::Table, GuildChannels::GuildId)
                            .to(Guilds::Table, Guilds::GuildId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_guild_channels_channel_type")
                            .from(GuildChannels::Table, GuildChannels::ChannelType)
                            .to(ChannelTypes::Table, ChannelTypes::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        // guild_idでの検索用インデックスを作成
        manager
            .create_index(
                Index::create()
                    .name("idx_guild_channels_guild_id")
                    .table(GuildChannels::Table)
                    .col(GuildChannels::GuildId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GuildChannels::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum GuildChannels {
    Table,
    GuildId,
    ChannelType,
    ChannelId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Guilds {
    Table,
    GuildId,
}

#[derive(DeriveIden)]
enum ChannelTypes {
    Table,
    Id,
}
