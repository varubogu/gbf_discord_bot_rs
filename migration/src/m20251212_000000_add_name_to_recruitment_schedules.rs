use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // battle_recruitment_schedules テーブルに name カラムを追加
        manager
            .alter_table(
                Table::alter()
                    .table((Alias::new("worker"), BattleRecruitmentSchedules::Table))
                    .add_column(
                        ColumnDef::new(BattleRecruitmentSchedules::Name)
                            .string()
                            .not_null()
                            .default("未設定"),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // name カラムを削除
        manager
            .alter_table(
                Table::alter()
                    .table((Alias::new("worker"), BattleRecruitmentSchedules::Table))
                    .drop_column(BattleRecruitmentSchedules::Name)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum BattleRecruitmentSchedules {
    Table,
    Name,
}
