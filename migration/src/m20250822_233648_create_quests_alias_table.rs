use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(QuestAlias::Table)
                    .if_not_exists()
                    .col(pk_auto(QuestAlias::Id))
                    .col(integer(QuestAlias::TargetId))
                    .col(string(QuestAlias::Alias))
                    .col(timestamp_with_time_zone(QuestAlias::CreatedAt))
                    .col(timestamp_with_time_zone(QuestAlias::UpdatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-quests_alias-target_id")
                            .from(QuestAlias::Table, QuestAlias::TargetId)
                            .to(Quest::Table, Quest::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(QuestAlias::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum QuestAlias {
    Table,
    Id,
    TargetId,
    Alias,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Quest {
    Table,
    Id,
}
