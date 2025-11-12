use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{DatabaseBackend, Statement};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // 既存の主キー制約を削除
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "ALTER TABLE quest_aliases DROP CONSTRAINT IF EXISTS quest_aliases_pkey",
        ))
        .await?;

        // idカラムをsequence_noにリネーム
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "ALTER TABLE quest_aliases RENAME COLUMN id TO sequence_no",
        ))
        .await?;

        // quest_idとsequence_noで複合主キーを作成（順序を正しく）
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "ALTER TABLE quest_aliases ADD CONSTRAINT quest_aliases_pkey PRIMARY KEY (quest_id, sequence_no)",
        ))
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // 複合主キーを削除
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "ALTER TABLE quest_aliases DROP CONSTRAINT IF EXISTS quest_aliases_pkey",
        ))
        .await?;

        // sequence_noカラムをidにリネーム
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "ALTER TABLE quest_aliases RENAME COLUMN sequence_no TO id",
        ))
        .await?;

        // 元の主キー(id, quest_id)に戻す
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "ALTER TABLE quest_aliases ADD CONSTRAINT quest_aliases_pkey PRIMARY KEY (id, quest_id)",
        ))
        .await?;

        Ok(())
    }
}
