use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // battle_recruitment_schedules テーブル作成
        manager
            .create_table(
                Table::create()
                    .table((Alias::new("worker"), BattleRecruitmentSchedules::Table))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(BattleRecruitmentSchedules::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(BattleRecruitmentSchedules::Name)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(BattleRecruitmentSchedules::GuildId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(BattleRecruitmentSchedules::ChannelId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(BattleRecruitmentSchedules::QuestId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(BattleRecruitmentSchedules::BattleStyleId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(BattleRecruitmentSchedules::QuestStartTime)
                            .time()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(BattleRecruitmentSchedules::RecruitStartDayOffset)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(BattleRecruitmentSchedules::RecruitStartTime)
                            .time()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(BattleRecruitmentSchedules::MaxParticipants)
                            .integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(BattleRecruitmentSchedules::Note)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(BattleRecruitmentSchedules::IsEnabled)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(BattleRecruitmentSchedules::CreatedBy)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        timestamp_with_time_zone(BattleRecruitmentSchedules::CreatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(BattleRecruitmentSchedules::UpdatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    // guild_id への外部キー制約
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_battle_recruitment_schedules_guild_id")
                            .from(
                                (Alias::new("worker"), BattleRecruitmentSchedules::Table),
                                BattleRecruitmentSchedules::GuildId,
                            )
                            .to((Alias::new("guild_master"), Guilds::Table), Guilds::GuildId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    // quest_id への外部キー制約
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_battle_recruitment_schedules_quest_id")
                            .from(
                                (Alias::new("worker"), BattleRecruitmentSchedules::Table),
                                BattleRecruitmentSchedules::QuestId,
                            )
                            .to((Alias::new("master"), Quests::Table), Quests::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    // battle_style_id への外部キー制約
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_battle_recruitment_schedules_battle_style_id")
                            .from(
                                (Alias::new("worker"), BattleRecruitmentSchedules::Table),
                                BattleRecruitmentSchedules::BattleStyleId,
                            )
                            .to((Alias::new("master"), BattleStyles::Table), BattleStyles::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        // インデックスを追加（検索性能向上）
        manager
            .create_index(
                Index::create()
                    .name("idx_battle_recruitment_schedules_guild_enabled")
                    .table((Alias::new("worker"), BattleRecruitmentSchedules::Table))
                    .col(BattleRecruitmentSchedules::GuildId)
                    .col(BattleRecruitmentSchedules::IsEnabled)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_battle_recruitment_schedules_created_by")
                    .table((Alias::new("worker"), BattleRecruitmentSchedules::Table))
                    .col(BattleRecruitmentSchedules::CreatedBy)
                    .to_owned(),
            )
            .await?;

        // battle_recruitment_schedule_days テーブル作成
        manager
            .create_table(
                Table::create()
                    .table((Alias::new("worker"), BattleRecruitmentScheduleDays::Table))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(BattleRecruitmentScheduleDays::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(BattleRecruitmentScheduleDays::ScheduleId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(BattleRecruitmentScheduleDays::DayOfWeek)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        timestamp_with_time_zone(BattleRecruitmentScheduleDays::CreatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(BattleRecruitmentScheduleDays::UpdatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    // UNIQUE制約: 同一スケジュールに同じ曜日を重複登録しない
                    .index(
                        Index::create()
                            .unique()
                            .name("idx_battle_recruitment_schedule_days_unique")
                            .col(BattleRecruitmentScheduleDays::ScheduleId)
                            .col(BattleRecruitmentScheduleDays::DayOfWeek),
                    )
                    // schedule_id への外部キー制約（CASCADE削除）
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_battle_recruitment_schedule_days_schedule_id")
                            .from(
                                (Alias::new("worker"), BattleRecruitmentScheduleDays::Table),
                                BattleRecruitmentScheduleDays::ScheduleId,
                            )
                            .to(
                                (Alias::new("worker"), BattleRecruitmentSchedules::Table),
                                BattleRecruitmentSchedules::Id,
                            )
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // インデックスを追加（検索性能向上）
        manager
            .create_index(
                Index::create()
                    .name("idx_battle_recruitment_schedule_days_schedule_id")
                    .table((Alias::new("worker"), BattleRecruitmentScheduleDays::Table))
                    .col(BattleRecruitmentScheduleDays::ScheduleId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // battle_recruitment_schedule_days テーブル削除
        manager
            .drop_table(
                Table::drop()
                    .table((Alias::new("worker"), BattleRecruitmentScheduleDays::Table))
                    .to_owned(),
            )
            .await?;

        // battle_recruitment_schedules テーブル削除
        manager
            .drop_table(
                Table::drop()
                    .table((Alias::new("worker"), BattleRecruitmentSchedules::Table))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

// テーブル識別子の定義
#[derive(DeriveIden)]
enum BattleRecruitmentSchedules {
    Table,
    Id,
    Name,
    GuildId,
    ChannelId,
    QuestId,
    BattleStyleId,
    QuestStartTime,
    RecruitStartDayOffset,
    RecruitStartTime,
    MaxParticipants,
    Note,
    IsEnabled,
    CreatedBy,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum BattleRecruitmentScheduleDays {
    Table,
    Id,
    ScheduleId,
    DayOfWeek,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Guilds {
    Table,
    GuildId,
}

#[derive(DeriveIden)]
enum Quests {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum BattleStyles {
    Table,
    Id,
}
