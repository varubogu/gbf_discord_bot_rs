use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{DatabaseBackend, Statement};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // battle_types テーブルを battle_styles にリネーム
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "ALTER TABLE battle_types RENAME TO battle_styles",
        ))
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // battle_styles テーブルを battle_types に戻す
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "ALTER TABLE battle_styles RENAME TO battle_types",
        ))
        .await?;

        Ok(())
    }
}
