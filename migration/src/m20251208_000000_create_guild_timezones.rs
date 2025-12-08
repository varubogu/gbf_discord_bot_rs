use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // guild_timezones テーブル作成
        manager
            .create_table(
                Table::create()
                    .table((Alias::new("guild_master"), GuildTimezones::Table))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(GuildTimezones::GuildId)
                            .big_integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(GuildTimezones::Timezone)
                            .string()
                            .not_null(),
                    )
                    .col(
                        timestamp_with_time_zone(GuildTimezones::CreatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(GuildTimezones::UpdatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // guild_timezones テーブル削除
        manager
            .drop_table(
                Table::drop()
                    .table((Alias::new("guild_master"), GuildTimezones::Table))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

// テーブル識別子の定義
#[derive(DeriveIden)]
enum GuildTimezones {
    Table,
    GuildId,
    Timezone,
    CreatedAt,
    UpdatedAt,
}
