use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. 汎用的な更新関数を作成
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE OR REPLACE FUNCTION update_updated_at_column()
                 RETURNS TRIGGER AS $$
                 BEGIN
                     NEW.updated_at = CURRENT_TIMESTAMP;
                     RETURN NEW;
                 END;
                 $$ language 'plpgsql';",
            )
            .await?;

        // 2. 各テーブルのcreated_atにデフォルト値を設定
        let tables = [
            "environments",
            "quests",
            "quest_aliases",
            "message_texts",
            "battle_recruitments",
        ];

        for table_name in tables.iter() {
            // created_atにデフォルト値を設定
            manager
                .get_connection()
                .execute_unprepared(&format!(
                    "ALTER TABLE {} ALTER COLUMN created_at SET DEFAULT CURRENT_TIMESTAMP;",
                    table_name
                ))
                .await?;

            // updated_atにデフォルト値を設定
            manager
                .get_connection()
                .execute_unprepared(&format!(
                    "ALTER TABLE {} ALTER COLUMN updated_at SET DEFAULT CURRENT_TIMESTAMP;",
                    table_name
                ))
                .await?;

            // updated_at自動更新トリガーを設定
            manager
                .get_connection()
                .execute_unprepared(&format!(
                    "CREATE TRIGGER update_{}_updated_at 
                     BEFORE UPDATE ON {} 
                     FOR EACH ROW 
                     EXECUTE FUNCTION update_updated_at_column();",
                    table_name, table_name
                ))
                .await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // トリガーを削除
        let tables = [
            "environments",
            "quests",
            "quest_aliases",
            "message_texts",
            "battle_recruitments",
        ];

        for table_name in tables.iter() {
            // トリガーを削除
            manager
                .get_connection()
                .execute_unprepared(&format!(
                    "DROP TRIGGER IF EXISTS update_{}_updated_at ON {};",
                    table_name, table_name
                ))
                .await?;

            // デフォルト値を削除
            manager
                .get_connection()
                .execute_unprepared(&format!(
                    "ALTER TABLE {} ALTER COLUMN created_at DROP DEFAULT;",
                    table_name
                ))
                .await?;

            manager
                .get_connection()
                .execute_unprepared(&format!(
                    "ALTER TABLE {} ALTER COLUMN updated_at DROP DEFAULT;",
                    table_name
                ))
                .await?;
        }

        // 更新関数を削除
        manager
            .get_connection()
            .execute_unprepared("DROP FUNCTION IF EXISTS update_updated_at_column();")
            .await?;

        Ok(())
    }
}
