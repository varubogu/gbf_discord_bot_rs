use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // guild_spreadsheet_imports テーブル作成
        manager
            .create_table(
                Table::create()
                    .table(GuildSpreadsheetImports::Table)
                    .if_not_exists()
                    .col(
                        big_integer(GuildSpreadsheetImports::GuildId)
                            .not_null()
                            .primary_key(),
                    )
                    .col(string(GuildSpreadsheetImports::SpreadsheetId).not_null())
                    .col(
                        timestamp_with_time_zone(GuildSpreadsheetImports::CreatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(GuildSpreadsheetImports::UpdatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // guild_spreadsheet_exports テーブル作成
        manager
            .create_table(
                Table::create()
                    .table(GuildSpreadsheetExports::Table)
                    .if_not_exists()
                    .col(
                        big_integer(GuildSpreadsheetExports::GuildId)
                            .not_null()
                            .primary_key(),
                    )
                    .col(string(GuildSpreadsheetExports::SpreadsheetId).not_null())
                    .col(
                        timestamp_with_time_zone(GuildSpreadsheetExports::CreatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(GuildSpreadsheetExports::UpdatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GuildSpreadsheetImports::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(GuildSpreadsheetExports::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum GuildSpreadsheetImports {
    Table,
    GuildId,
    SpreadsheetId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum GuildSpreadsheetExports {
    Table,
    GuildId,
    SpreadsheetId,
    CreatedAt,
    UpdatedAt,
}
