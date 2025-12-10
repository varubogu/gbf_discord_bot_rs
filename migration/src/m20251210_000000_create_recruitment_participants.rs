use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // recruitment_participants テーブル作成
        manager
            .create_table(
                Table::create()
                    .table((Alias::new("worker"), RecruitmentParticipants::Table))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(RecruitmentParticipants::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(RecruitmentParticipants::RecruitmentId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RecruitmentParticipants::UserId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RecruitmentParticipants::ElementId)
                            .integer()
                            .null(),
                    )
                    .col(
                        timestamp_with_time_zone(RecruitmentParticipants::ParticipatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(RecruitmentParticipants::CreatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(RecruitmentParticipants::UpdatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    // UNIQUE制約: 同一募集で同一ユーザーが同一属性に複数回参加できないようにする
                    .index(
                        Index::create()
                            .unique()
                            .name("idx_recruitment_participants_unique")
                            .col(RecruitmentParticipants::RecruitmentId)
                            .col(RecruitmentParticipants::UserId)
                            .col(RecruitmentParticipants::ElementId),
                    )
                    // recruitment_id への外部キー制約
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_recruitment_participants_recruitment_id")
                            .from(
                                (Alias::new("worker"), RecruitmentParticipants::Table),
                                RecruitmentParticipants::RecruitmentId,
                            )
                            .to(
                                (Alias::new("worker"), BattleRecruitments::Table),
                                BattleRecruitments::Id,
                            )
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    // element_id への外部キー制約
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_recruitment_participants_element_id")
                            .from(
                                (Alias::new("worker"), RecruitmentParticipants::Table),
                                RecruitmentParticipants::ElementId,
                            )
                            .to((Alias::new("master"), Elements::Table), Elements::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        // インデックスを追加（検索性能向上）
        manager
            .create_index(
                Index::create()
                    .name("idx_recruitment_participants_recruitment_id")
                    .table((Alias::new("worker"), RecruitmentParticipants::Table))
                    .col(RecruitmentParticipants::RecruitmentId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_recruitment_participants_user_id")
                    .table((Alias::new("worker"), RecruitmentParticipants::Table))
                    .col(RecruitmentParticipants::UserId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // recruitment_participants テーブル削除
        manager
            .drop_table(
                Table::drop()
                    .table((Alias::new("worker"), RecruitmentParticipants::Table))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

// テーブル識別子の定義
#[derive(DeriveIden)]
enum RecruitmentParticipants {
    Table,
    Id,
    RecruitmentId,
    UserId,
    ElementId,
    ParticipatedAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum BattleRecruitments {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Elements {
    Table,
    Id,
}
