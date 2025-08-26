use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Environments テーブル作成
        manager
            .create_table(
                Table::create()
                    .table(Environments::Table)
                    .if_not_exists()
                    .col(pk_auto(Environments::Id))
                    .col(string(Environments::Key))
                    .col(string(Environments::Value))
                    .col(timestamp_with_time_zone(Environments::CreatedAt))
                    .col(timestamp_with_time_zone(Environments::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        // Quests テーブル作成
        manager
            .create_table(
                Table::create()
                    .table(Quests::Table)
                    .if_not_exists()
                    .col(pk_auto(Quests::Id))
                    .col(integer(Quests::TargetId))
                    .col(string(Quests::QuestName))
                    .col(integer(Quests::DefaultBattleType))
                    .col(timestamp_with_time_zone(Quests::CreatedAt))
                    .col(timestamp_with_time_zone(Quests::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        // Quest Aliases テーブル作成
        manager
            .create_table(
                Table::create()
                    .table(QuestAliases::Table)
                    .if_not_exists()
                    .col(pk_auto(QuestAliases::Id))
                    .col(integer(QuestAliases::TargetId))
                    .col(string(QuestAliases::Alias))
                    .col(timestamp_with_time_zone(QuestAliases::CreatedAt))
                    .col(timestamp_with_time_zone(QuestAliases::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        // Quest AliasesからQuestsへの外部キー制約追加
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_quest_aliases_target_id")
                    .from(QuestAliases::Table, QuestAliases::TargetId)
                    .to(Quests::Table, Quests::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        // Message Texts テーブル作成
        manager
            .create_table(
                Table::create()
                    .table(MessageTexts::Table)
                    .if_not_exists()
                    .col(pk_auto(MessageTexts::Id))
                    .col(big_integer(MessageTexts::GuildId))
                    .col(string(MessageTexts::MessageId))
                    .col(string(MessageTexts::MessageJp))
                    .col(string_null(MessageTexts::MessageEn))
                    .col(timestamp_with_time_zone(MessageTexts::CreatedAt))
                    .col(timestamp_with_time_zone(MessageTexts::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        // Battle Recruitments テーブル作成
        manager
            .create_table(
                Table::create()
                    .table(BattleRecruitments::Table)
                    .if_not_exists()
                    .col(pk_auto(BattleRecruitments::Id))
                    .col(big_integer(BattleRecruitments::GuildId))
                    .col(big_integer(BattleRecruitments::ChannelId))
                    .col(big_integer(BattleRecruitments::MessageId))
                    .col(integer(BattleRecruitments::TargetId))
                    .col(integer(BattleRecruitments::BattleTypeId))
                    .col(timestamp_with_time_zone(BattleRecruitments::ExpiryDate))
                    .col(big_integer_null(BattleRecruitments::RecruitEndMessageId))
                    .col(timestamp_with_time_zone(BattleRecruitments::CreatedAt))
                    .col(timestamp_with_time_zone(BattleRecruitments::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(BattleRecruitments::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(MessageTexts::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(QuestAliases::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Quests::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Environments::Table).to_owned())
            .await?;

        Ok(())
    }
}

// テーブル識別子の定義
#[derive(DeriveIden)]
enum Environments {
    Table,
    Id,
    Key,
    Value,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Quests {
    Table,
    Id,
    TargetId,
    QuestName,
    DefaultBattleType,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum QuestAliases {
    Table,
    Id,
    TargetId,
    Alias,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum MessageTexts {
    Table,
    Id,
    GuildId,
    MessageId,
    MessageJp,
    MessageEn,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum BattleRecruitments {
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
