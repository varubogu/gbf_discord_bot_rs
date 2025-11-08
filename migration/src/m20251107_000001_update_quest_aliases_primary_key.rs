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

        // idカラムのDEFAULT値を削除（シーケンスへの依存を解除）
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "ALTER TABLE quest_aliases ALTER COLUMN id DROP DEFAULT",
        ))
        .await?;

        // シーケンスを削除（CASCADE オプションで依存も削除）
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "DROP SEQUENCE IF EXISTS quest_aliases_id_seq CASCADE",
        ))
        .await?;

        // idとquest_idで複合主キーを作成
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "ALTER TABLE quest_aliases ADD CONSTRAINT quest_aliases_pkey PRIMARY KEY (id, quest_id)",
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

        // idのみの主キーに戻す
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "CREATE SEQUENCE IF NOT EXISTS quest_aliases_id_seq OWNED BY quest_aliases.id",
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "ALTER TABLE quest_aliases ALTER COLUMN id SET DEFAULT nextval('quest_aliases_id_seq')",
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "ALTER TABLE quest_aliases ADD CONSTRAINT quest_aliases_pkey PRIMARY KEY (id)",
        ))
        .await?;

        Ok(())
    }
}
