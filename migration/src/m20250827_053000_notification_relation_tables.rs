use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // notification_rel_event_schedules テーブル作成
        manager
            .create_table(
                Table::create()
                    .table(NotificationRelEventSchedules::Table)
                    .if_not_exists()
                    .col(integer(NotificationRelEventSchedules::EventScheduleId))
                    .col(integer_null(
                        NotificationRelEventSchedules::EventScheduleDetailId,
                    ))
                    .col(integer(NotificationRelEventSchedules::NotificationId))
                    .col(timestamp_with_time_zone(
                        NotificationRelEventSchedules::CreatedAt,
                    ))
                    // 複合主キー設定
                    .primary_key(
                        Index::create()
                            .name("pk_notification_rel_event_schedules")
                            .col(NotificationRelEventSchedules::EventScheduleId)
                            .col(NotificationRelEventSchedules::NotificationId),
                    )
                    .to_owned(),
            )
            .await?;

        // notification_rel_battle_recruitments テーブル作成
        manager
            .create_table(
                Table::create()
                    .table(NotificationRelBattleRecruitments::Table)
                    .if_not_exists()
                    .col(integer(NotificationRelBattleRecruitments::RecruitId))
                    .col(integer(NotificationRelBattleRecruitments::NotificationId))
                    .col(timestamp_with_time_zone(
                        NotificationRelBattleRecruitments::CreatedAt,
                    ))
                    // 複合主キー設定
                    .primary_key(
                        Index::create()
                            .name("pk_notification_rel_battle_recruitments")
                            .col(NotificationRelBattleRecruitments::RecruitId)
                            .col(NotificationRelBattleRecruitments::NotificationId),
                    )
                    .to_owned(),
            )
            .await?;

        // 外部キー制約の追加
        // notification_rel_event_schedules -> notifications
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_notification_rel_event_schedules_notification_id")
                    .from(
                        NotificationRelEventSchedules::Table,
                        NotificationRelEventSchedules::NotificationId,
                    )
                    .to(Notifications::Table, Notifications::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        // notification_rel_event_schedules -> event_schedules
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_notification_rel_event_schedules_event_schedule_id")
                    .from(
                        NotificationRelEventSchedules::Table,
                        NotificationRelEventSchedules::EventScheduleId,
                    )
                    .to(EventSchedules::Table, EventSchedules::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        // notification_rel_event_schedules -> event_schedule_details
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_notification_rel_event_schedules_event_schedule_detail_id")
                    .from(
                        NotificationRelEventSchedules::Table,
                        NotificationRelEventSchedules::EventScheduleDetailId,
                    )
                    .to(EventScheduleDetails::Table, EventScheduleDetails::Id)
                    .on_delete(ForeignKeyAction::SetNull)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        // notification_rel_battle_recruitments -> notifications
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_notification_rel_battle_recruitments_notification_id")
                    .from(
                        NotificationRelBattleRecruitments::Table,
                        NotificationRelBattleRecruitments::NotificationId,
                    )
                    .to(Notifications::Table, Notifications::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        // notification_rel_battle_recruitments -> battle_recruitments
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_notification_rel_battle_recruitments_recruit_id")
                    .from(
                        NotificationRelBattleRecruitments::Table,
                        NotificationRelBattleRecruitments::RecruitId,
                    )
                    .to(BattleRecruitments::Table, BattleRecruitments::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(NotificationRelBattleRecruitments::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(NotificationRelEventSchedules::Table)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

// テーブル識別子の定義
#[derive(DeriveIden)]
enum NotificationRelEventSchedules {
    Table,
    EventScheduleId,
    EventScheduleDetailId,
    NotificationId,
    CreatedAt,
}

#[derive(DeriveIden)]
enum NotificationRelBattleRecruitments {
    Table,
    RecruitId,
    NotificationId,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Notifications {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum EventSchedules {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum EventScheduleDetails {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum BattleRecruitments {
    Table,
    Id,
}
