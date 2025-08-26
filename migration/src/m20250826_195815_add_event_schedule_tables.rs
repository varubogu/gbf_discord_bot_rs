use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Event Schedules テーブル作成
        manager
            .create_table(
                Table::create()
                    .table(EventSchedules::Table)
                    .if_not_exists()
                    .col(pk_auto(EventSchedules::Id))
                    .col(string(EventSchedules::EventType))
                    .col(big_integer(EventSchedules::EventCount))
                    .col(string(EventSchedules::Profile))
                    .col(integer(EventSchedules::WeakAttribute))
                    .col(timestamp_with_time_zone(EventSchedules::StartAt))
                    .col(timestamp_with_time_zone(EventSchedules::EndAt))
                    .col(timestamp_with_time_zone(EventSchedules::CreatedAt))
                    .col(timestamp_with_time_zone(EventSchedules::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        // Event Schedules のユニーク制約追加
        manager
            .create_index(
                Index::create()
                    .name("unique_event_type_count")
                    .table(EventSchedules::Table)
                    .col(EventSchedules::EventType)
                    .col(EventSchedules::EventCount)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Event Schedule Details テーブル作成
        manager
            .create_table(
                Table::create()
                    .table(EventScheduleDetails::Table)
                    .if_not_exists()
                    .col(pk_auto(EventScheduleDetails::Id))
                    .col(string(EventScheduleDetails::Profile))
                    .col(string(EventScheduleDetails::StartDayRelative))
                    .col(string(EventScheduleDetails::Time))
                    .col(string(EventScheduleDetails::ScheduleName))
                    .col(string(EventScheduleDetails::MessageId))
                    .col(big_integer(EventScheduleDetails::GuildId))
                    .col(big_integer(EventScheduleDetails::ChannelId))
                    .col(string(EventScheduleDetails::Reactions))
                    .col(timestamp_with_time_zone(EventScheduleDetails::CreatedAt))
                    .col(timestamp_with_time_zone(EventScheduleDetails::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        // Last Process Times テーブル作成
        manager
            .create_table(
                Table::create()
                    .table(LastProcessTimes::Table)
                    .if_not_exists()
                    .col(integer(LastProcessTimes::ProcessType).primary_key())
                    .col(timestamp_with_time_zone_null(LastProcessTimes::ExecuteTime))
                    .col(string(LastProcessTimes::Memo))
                    .col(timestamp_with_time_zone(LastProcessTimes::CreatedAt))
                    .col(timestamp_with_time_zone(LastProcessTimes::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        // Schedules テーブル作成
        manager
            .create_table(
                Table::create()
                    .table(Schedules::Table)
                    .if_not_exists()
                    .col(pk_auto(Schedules::Id))
                    .col(integer_null(Schedules::ParentScheduleId))
                    .col(integer_null(Schedules::ParentScheduleDetailId))
                    .col(timestamp_with_time_zone(Schedules::ScheduleDatetime))
                    .col(big_integer(Schedules::GuildId))
                    .col(big_integer(Schedules::ChannelId))
                    .col(string(Schedules::MessageId))
                    .col(timestamp_with_time_zone(Schedules::CreatedAt))
                    .col(timestamp_with_time_zone(Schedules::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        // Schedules から Event Schedules への外部キー制約
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_schedules_parent_schedule_id")
                    .from(Schedules::Table, Schedules::ParentScheduleId)
                    .to(EventSchedules::Table, EventSchedules::Id)
                    .on_delete(ForeignKeyAction::SetNull)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        // Schedules から Event Schedule Details への外部キー制約
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_schedules_parent_schedule_detail_id")
                    .from(Schedules::Table, Schedules::ParentScheduleDetailId)
                    .to(EventScheduleDetails::Table, EventScheduleDetails::Id)
                    .on_delete(ForeignKeyAction::SetNull)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Schedules::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(LastProcessTimes::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(EventScheduleDetails::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(EventSchedules::Table).to_owned())
            .await?;

        Ok(())
    }
}

// テーブル識別子の定義
#[derive(DeriveIden)]
enum EventSchedules {
    Table,
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
enum EventScheduleDetails {
    Table,
    Id,
    Profile,
    StartDayRelative,
    Time,
    ScheduleName,
    MessageId,
    GuildId,
    ChannelId,
    Reactions,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum LastProcessTimes {
    Table,
    ProcessType,
    ExecuteTime,
    Memo,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Schedules {
    Table,
    Id,
    ParentScheduleId,
    ParentScheduleDetailId,
    ScheduleDatetime,
    GuildId,
    ChannelId,
    MessageId,
    CreatedAt,
    UpdatedAt,
}
