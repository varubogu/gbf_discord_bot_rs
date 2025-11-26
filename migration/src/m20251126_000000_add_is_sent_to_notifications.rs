use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // notifications テーブルに is_sent カラムを追加
        manager
            .alter_table(
                Table::alter()
                    .table(Notifications::Table)
                    .add_column(
                        ColumnDef::new(Notifications::IsSent)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // is_sent カラムを削除
        manager
            .alter_table(
                Table::alter()
                    .table(Notifications::Table)
                    .drop_column(Notifications::IsSent)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Notifications {
    Table,
    IsSent,
}
