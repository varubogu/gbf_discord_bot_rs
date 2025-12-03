use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // guild_environments テーブル作成
        manager
            .create_table(
                Table::create()
                    .table((Alias::new("guild_master"), GuildEnvironments::Table))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(GuildEnvironments::GuildId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GuildEnvironments::Key)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GuildEnvironments::Value)
                            .string()
                            .not_null(),
                    )
                    .col(
                        timestamp_with_time_zone(GuildEnvironments::CreatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(GuildEnvironments::UpdatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .primary_key(
                        Index::create()
                            .col(GuildEnvironments::GuildId)
                            .col(GuildEnvironments::Key),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_guild_environments_guild_id")
                            .from(
                                (Alias::new("guild_master"), GuildEnvironments::Table),
                                GuildEnvironments::GuildId,
                            )
                            .to((Alias::new("guild_master"), Guilds::Table), Guilds::GuildId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // guild_event_schedules テーブル作成
        manager
            .create_table(
                Table::create()
                    .table((Alias::new("guild_master"), GuildEventSchedules::Table))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(GuildEventSchedules::GuildId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GuildEventSchedules::Id)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GuildEventSchedules::EventType)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GuildEventSchedules::EventCount)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GuildEventSchedules::Profile)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GuildEventSchedules::WeakAttribute)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GuildEventSchedules::StartAt)
                            .timestamp()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GuildEventSchedules::EndAt)
                            .timestamp()
                            .not_null(),
                    )
                    .col(
                        timestamp_with_time_zone(GuildEventSchedules::CreatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(GuildEventSchedules::UpdatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .primary_key(
                        Index::create()
                            .col(GuildEventSchedules::GuildId)
                            .col(GuildEventSchedules::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_guild_event_schedules_guild_id")
                            .from(
                                (Alias::new("guild_master"), GuildEventSchedules::Table),
                                GuildEventSchedules::GuildId,
                            )
                            .to((Alias::new("guild_master"), Guilds::Table), Guilds::GuildId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // guild_event_schedule_details テーブル作成
        manager
            .create_table(
                Table::create()
                    .table((Alias::new("guild_master"), GuildEventScheduleDetails::Table))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(GuildEventScheduleDetails::GuildId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GuildEventScheduleDetails::Id)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GuildEventScheduleDetails::Profile)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GuildEventScheduleDetails::StartDayRelative)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GuildEventScheduleDetails::Time)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GuildEventScheduleDetails::ScheduleName)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GuildEventScheduleDetails::MessageTextId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GuildEventScheduleDetails::NotificationChannelType)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GuildEventScheduleDetails::Reactions)
                            .string()
                            .not_null(),
                    )
                    .col(
                        timestamp_with_time_zone(GuildEventScheduleDetails::CreatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(GuildEventScheduleDetails::UpdatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .primary_key(
                        Index::create()
                            .col(GuildEventScheduleDetails::GuildId)
                            .col(GuildEventScheduleDetails::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_guild_event_schedule_details_guild_id")
                            .from(
                                (Alias::new("guild_master"), GuildEventScheduleDetails::Table),
                                GuildEventScheduleDetails::GuildId,
                            )
                            .to((Alias::new("guild_master"), Guilds::Table), Guilds::GuildId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // guild_message_texts テーブル作成
        manager
            .create_table(
                Table::create()
                    .table((Alias::new("guild_master"), GuildMessageTexts::Table))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(GuildMessageTexts::GuildId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GuildMessageTexts::Id)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GuildMessageTexts::MessageJp)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GuildMessageTexts::MessageEn)
                            .string()
                            .null(),
                    )
                    .col(
                        timestamp_with_time_zone(GuildMessageTexts::CreatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(GuildMessageTexts::UpdatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .primary_key(
                        Index::create()
                            .col(GuildMessageTexts::GuildId)
                            .col(GuildMessageTexts::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_guild_message_texts_guild_id")
                            .from(
                                (Alias::new("guild_master"), GuildMessageTexts::Table),
                                GuildMessageTexts::GuildId,
                            )
                            .to((Alias::new("guild_master"), Guilds::Table), Guilds::GuildId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // guild_last_process_times テーブル作成
        manager
            .create_table(
                Table::create()
                    .table((Alias::new("worker"), GuildLastProcessTimes::Table))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(GuildLastProcessTimes::GuildId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GuildLastProcessTimes::ProcessType)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GuildLastProcessTimes::ExecuteTime)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(GuildLastProcessTimes::Memo)
                            .string()
                            .not_null(),
                    )
                    .col(
                        timestamp_with_time_zone(GuildLastProcessTimes::CreatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(GuildLastProcessTimes::UpdatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .primary_key(
                        Index::create()
                            .col(GuildLastProcessTimes::GuildId)
                            .col(GuildLastProcessTimes::ProcessType),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_guild_last_process_times_guild_id")
                            .from(
                                (Alias::new("worker"), GuildLastProcessTimes::Table),
                                GuildLastProcessTimes::GuildId,
                            )
                            .to((Alias::new("guild_master"), Guilds::Table), Guilds::GuildId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 外部キー制約を持つテーブルから順に削除
        manager
            .drop_table(
                Table::drop()
                    .table((Alias::new("worker"), GuildLastProcessTimes::Table))
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table((Alias::new("guild_master"), GuildMessageTexts::Table))
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table((Alias::new("guild_master"), GuildEventScheduleDetails::Table))
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table((Alias::new("guild_master"), GuildEventSchedules::Table))
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table((Alias::new("guild_master"), GuildEnvironments::Table))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

// テーブル識別子の定義
#[derive(DeriveIden)]
enum GuildEnvironments {
    Table,
    GuildId,
    Key,
    Value,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum GuildEventSchedules {
    Table,
    GuildId,
    Id,
    EventType,
    EventCount,
    Profile,
    WeakAttribute,
    StartAt,
    EndAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum GuildEventScheduleDetails {
    Table,
    GuildId,
    Id,
    Profile,
    StartDayRelative,
    Time,
    ScheduleName,
    MessageTextId,
    NotificationChannelType,
    Reactions,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum GuildMessageTexts {
    Table,
    GuildId,
    Id,
    MessageJp,
    MessageEn,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum GuildLastProcessTimes {
    Table,
    GuildId,
    ProcessType,
    ExecuteTime,
    Memo,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Guilds {
    Table,
    GuildId,
}
