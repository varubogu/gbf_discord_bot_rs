use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // テーブル存在確認用SQL（COUNT を使用してエラーを回避）
        let timezones_result = conn
            .query_one_raw(sea_orm::Statement::from_string(
                manager.get_database_backend(),
                "SELECT COUNT(*) as count FROM information_schema.tables
                 WHERE table_schema = 'guild_master'
                 AND table_name = 'guild_timezones'".to_string(),
            ))
            .await?;

        let settings_result = conn
            .query_one_raw(sea_orm::Statement::from_string(
                manager.get_database_backend(),
                "SELECT COUNT(*) as count FROM information_schema.tables
                 WHERE table_schema = 'guild_master'
                 AND table_name = 'guild_settings'".to_string(),
            ))
            .await?;

        let timezones_count: i64 = timezones_result.unwrap().try_get("", "count")?;
        let settings_count: i64 = settings_result.unwrap().try_get("", "count")?;

        let timezones_exists = timezones_count > 0;
        let settings_exists = settings_count > 0;

        if timezones_exists {
            // guild_timezones が存在する場合（初回マイグレーション）
            // 1. locale 列を追加（生SQLで実行）
            conn.execute_unprepared(
                "ALTER TABLE guild_master.guild_timezones
                 ADD COLUMN locale VARCHAR(10) NOT NULL DEFAULT 'ja'"
            )
            .await?;

            // 2. guild_settings にリネーム（生SQLで実行）
            conn.execute_unprepared(
                "ALTER TABLE guild_master.guild_timezones
                 RENAME TO guild_settings"
            )
            .await?;
        } else if settings_exists {
            // guild_settings が既に存在する場合（リネーム済み）
            // locale 列の存在確認
            let locale_result = conn
                .query_one_raw(sea_orm::Statement::from_string(
                    manager.get_database_backend(),
                    "SELECT COUNT(*) as count FROM information_schema.columns
                     WHERE table_schema = 'guild_master'
                     AND table_name = 'guild_settings'
                     AND column_name = 'locale'".to_string(),
                ))
                .await?;

            let locale_count: i64 = locale_result.unwrap().try_get("", "count")?;
            let locale_exists = locale_count > 0;

            if !locale_exists {
                // locale 列がまだ存在しない場合のみ追加（生SQLで実行）
                conn.execute_unprepared(
                    "ALTER TABLE guild_master.guild_settings
                     ADD COLUMN locale VARCHAR(10) NOT NULL DEFAULT 'ja'"
                )
                .await?;
            }
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // 1. guild_settings テーブルを guild_timezones に戻す（生SQLで実行）
        conn.execute_unprepared(
            "ALTER TABLE guild_master.guild_settings
             RENAME TO guild_timezones"
        )
        .await?;

        // 2. locale 列を削除（生SQLで実行）
        conn.execute_unprepared(
            "ALTER TABLE guild_master.guild_timezones
             DROP COLUMN locale"
        )
        .await?;

        Ok(())
    }
}
