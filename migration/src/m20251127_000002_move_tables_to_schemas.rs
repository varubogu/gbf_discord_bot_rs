use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // master スキーマへテーブルを移動
        let master_tables = vec![
            "quests",
            "quest_aliases",
            "battle_styles",
            "elements",
            "channel_types",
            "event_schedules",
            "event_schedule_details",
            "message_texts",
            "environments",
        ];

        for table in master_tables {
            conn.execute_unprepared(&format!(
                "ALTER TABLE IF EXISTS public.{} SET SCHEMA master",
                table
            ))
            .await?;
        }

        // guild_master スキーマへテーブルを移動
        let guild_master_tables = vec![
            "guilds",
            "guild_channels",
            "guild_spreadsheet_exports",
            "guild_spreadsheet_imports",
        ];

        for table in guild_master_tables {
            conn.execute_unprepared(&format!(
                "ALTER TABLE IF EXISTS public.{} SET SCHEMA guild_master",
                table
            ))
            .await?;
        }

        // worker スキーマへテーブルを移動
        let worker_tables = vec![
            "battle_recruitments",
            "notifications",
            "notification_rel_battle_recruitments",
            "notification_rel_event_schedules",
            "last_process_times",
        ];

        for table in worker_tables {
            conn.execute_unprepared(&format!(
                "ALTER TABLE IF EXISTS public.{} SET SCHEMA worker",
                table
            ))
            .await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // すべてのテーブルを public スキーマに戻す
        let all_tables = vec![
            // master から
            ("master", "quests"),
            ("master", "quest_aliases"),
            ("master", "battle_styles"),
            ("master", "elements"),
            ("master", "channel_types"),
            ("master", "event_schedules"),
            ("master", "event_schedule_details"),
            ("master", "message_texts"),
            ("master", "environments"),
            // guild_master から
            ("guild_master", "guilds"),
            ("guild_master", "guild_channels"),
            ("guild_master", "guild_spreadsheet_exports"),
            ("guild_master", "guild_spreadsheet_imports"),
            // worker から
            ("worker", "battle_recruitments"),
            ("worker", "notifications"),
            ("worker", "notification_rel_battle_recruitments"),
            ("worker", "notification_rel_event_schedules"),
            ("worker", "last_process_times"),
        ];

        for (schema, table) in all_tables {
            conn.execute_unprepared(&format!(
                "ALTER TABLE IF EXISTS {}.{} SET SCHEMA public",
                schema, table
            ))
            .await?;
        }

        Ok(())
    }
}
