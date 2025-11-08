use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{DatabaseBackend, Statement};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // channel_type → id に変更
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"
DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_name = 'channel_types'
          AND column_name = 'channel_type'
    ) THEN
        ALTER TABLE channel_types RENAME COLUMN channel_type TO id;
    END IF;
END $$;
"#,
        ))
        .await?;

        // channel_type_name → name に変更
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"
DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_name = 'channel_types'
          AND column_name = 'channel_type_name'
    ) THEN
        ALTER TABLE channel_types RENAME COLUMN channel_type_name TO name;
    END IF;
END $$;
"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "ALTER TABLE channel_types ALTER COLUMN id DROP DEFAULT",
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "DROP SEQUENCE IF EXISTS channel_types_id_seq",
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "ALTER TABLE channel_types ALTER COLUMN id SET NOT NULL",
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "ALTER TABLE channel_types ALTER COLUMN name SET NOT NULL",
        ))
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // name → channel_type_name に戻す
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"
DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_name = 'channel_types'
          AND column_name = 'name'
    ) THEN
        ALTER TABLE channel_types RENAME COLUMN name TO channel_type_name;
    END IF;
END $$;
"#,
        ))
        .await?;

        // id → channel_type に戻す
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"
DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_name = 'channel_types'
          AND column_name = 'id'
    ) THEN
        ALTER TABLE channel_types RENAME COLUMN id TO channel_type;
    END IF;
END $$;
"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "CREATE SEQUENCE IF NOT EXISTS channel_types_id_seq OWNED BY channel_types.channel_type",
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "ALTER TABLE channel_types ALTER COLUMN channel_type SET DEFAULT nextval('channel_types_id_seq')",
        ))
        .await?;

        Ok(())
    }
}
