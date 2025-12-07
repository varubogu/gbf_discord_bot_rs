use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // all_recruitment_notification_roles テーブル作成
        manager
            .create_table(
                Table::create()
                    .table((
                        Alias::new("guild_master"),
                        AllRecruitmentNotificationRoles::Table,
                    ))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AllRecruitmentNotificationRoles::GuildId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AllRecruitmentNotificationRoles::Seq)
                            .integer()
                            .not_null()
                            .auto_increment(),
                    )
                    .col(
                        ColumnDef::new(AllRecruitmentNotificationRoles::RoleId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        timestamp_with_time_zone(AllRecruitmentNotificationRoles::CreatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(AllRecruitmentNotificationRoles::UpdatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .primary_key(
                        Index::create()
                            .col(AllRecruitmentNotificationRoles::GuildId)
                            .col(AllRecruitmentNotificationRoles::Seq),
                    )
                    .to_owned(),
            )
            .await?;

        // all_recruitment_notification_roles の UNIQUE 制約を追加
        manager
            .create_index(
                Index::create()
                    .name("uq_all_recruitment_notification_roles_guild_role")
                    .table((
                        Alias::new("guild_master"),
                        AllRecruitmentNotificationRoles::Table,
                    ))
                    .col(AllRecruitmentNotificationRoles::GuildId)
                    .col(AllRecruitmentNotificationRoles::RoleId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // quest_recruitment_notification_roles テーブル作成
        manager
            .create_table(
                Table::create()
                    .table((
                        Alias::new("guild_master"),
                        QuestRecruitmentNotificationRoles::Table,
                    ))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(QuestRecruitmentNotificationRoles::GuildId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(QuestRecruitmentNotificationRoles::QuestId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(QuestRecruitmentNotificationRoles::Seq)
                            .integer()
                            .not_null()
                            .auto_increment(),
                    )
                    .col(
                        ColumnDef::new(QuestRecruitmentNotificationRoles::RoleId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        timestamp_with_time_zone(QuestRecruitmentNotificationRoles::CreatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(QuestRecruitmentNotificationRoles::UpdatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .primary_key(
                        Index::create()
                            .col(QuestRecruitmentNotificationRoles::GuildId)
                            .col(QuestRecruitmentNotificationRoles::QuestId)
                            .col(QuestRecruitmentNotificationRoles::Seq),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_quest_recruitment_notification_roles_quest_id")
                            .from(
                                (
                                    Alias::new("guild_master"),
                                    QuestRecruitmentNotificationRoles::Table,
                                ),
                                QuestRecruitmentNotificationRoles::QuestId,
                            )
                            .to((Alias::new("master"), Quests::Table), Quests::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // quest_recruitment_notification_roles の UNIQUE 制約を追加
        manager
            .create_index(
                Index::create()
                    .name("uq_quest_recruitment_notification_roles_guild_quest_role")
                    .table((
                        Alias::new("guild_master"),
                        QuestRecruitmentNotificationRoles::Table,
                    ))
                    .col(QuestRecruitmentNotificationRoles::GuildId)
                    .col(QuestRecruitmentNotificationRoles::QuestId)
                    .col(QuestRecruitmentNotificationRoles::RoleId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // インデックスを追加（検索性能向上のため）
        manager
            .create_index(
                Index::create()
                    .name("idx_all_recruitment_notification_roles_guild_id")
                    .table((
                        Alias::new("guild_master"),
                        AllRecruitmentNotificationRoles::Table,
                    ))
                    .col(AllRecruitmentNotificationRoles::GuildId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_quest_recruitment_notification_roles_guild_quest")
                    .table((
                        Alias::new("guild_master"),
                        QuestRecruitmentNotificationRoles::Table,
                    ))
                    .col(QuestRecruitmentNotificationRoles::GuildId)
                    .col(QuestRecruitmentNotificationRoles::QuestId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // テーブルを削除（外部キー制約を持つテーブルから先に削除）
        manager
            .drop_table(
                Table::drop()
                    .table((
                        Alias::new("guild_master"),
                        QuestRecruitmentNotificationRoles::Table,
                    ))
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table((
                        Alias::new("guild_master"),
                        AllRecruitmentNotificationRoles::Table,
                    ))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

// テーブル識別子の定義
#[derive(DeriveIden)]
enum AllRecruitmentNotificationRoles {
    Table,
    GuildId,
    Seq,
    RoleId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum QuestRecruitmentNotificationRoles {
    Table,
    GuildId,
    QuestId,
    Seq,
    RoleId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Quests {
    Table,
    Id,
}
