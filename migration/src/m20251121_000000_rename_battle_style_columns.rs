use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{DatabaseBackend, Statement};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // battle_recruitments.battle_type_id -> battle_recruitments.battle_style_id
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "ALTER TABLE battle_recruitments RENAME COLUMN battle_type_id TO battle_style_id",
        ))
        .await?;

        // quests.default_battle_style -> quests.default_battle_style_id
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "ALTER TABLE quests RENAME COLUMN default_battle_style TO default_battle_style_id",
        ))
        .await?;

        // quests.available_battle_styles -> quests.available_battle_style_ids
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "ALTER TABLE quests RENAME COLUMN available_battle_styles TO available_battle_style_ids",
        ))
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // battle_recruitments.battle_style_id -> battle_recruitments.battle_type_id
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "ALTER TABLE battle_recruitments RENAME COLUMN battle_style_id TO battle_type_id",
        ))
        .await?;

        // quests.default_battle_style_id -> quests.default_battle_style
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "ALTER TABLE quests RENAME COLUMN default_battle_style_id TO default_battle_style",
        ))
        .await?;

        // quests.available_battle_style_ids -> quests.available_battle_styles
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "ALTER TABLE quests RENAME COLUMN available_battle_style_ids TO available_battle_styles",
        ))
        .await?;

        Ok(())
    }
}
