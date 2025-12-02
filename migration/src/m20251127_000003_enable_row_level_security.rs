use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // guild_master スキーマのテーブルに RLS を有効化
        let guild_master_tables = vec![
            "guilds",
            "guild_channels",
            "guild_spreadsheet_exports",
            "guild_spreadsheet_imports",
        ];

        for table in &guild_master_tables {
            // RLS 有効化
            conn.execute_unprepared(&format!(
                "ALTER TABLE guild_master.{} ENABLE ROW LEVEL SECURITY",
                table
            ))
            .await?;

            // gbf_bot_guild ロール用のポリシー作成
            conn.execute_unprepared(&format!(
                "CREATE POLICY {}_guild_policy ON guild_master.{} \
                 FOR ALL TO gbf_bot_guild \
                 USING (guild_id = current_setting('app.current_guild_id')::bigint)",
                table, table
            ))
            .await?;
        }

        // worker スキーマのテーブルに RLS を有効化
        let worker_tables = vec![
            "battle_recruitments",
            "notifications",
            "notification_rel_battle_recruitments",
            "notification_rel_event_schedules",
            "last_process_times",
        ];

        for table in &worker_tables {
            // RLS 有効化
            conn.execute_unprepared(&format!(
                "ALTER TABLE worker.{} ENABLE ROW LEVEL SECURITY",
                table
            ))
            .await?;

            // guild_id カラムを持つテーブルのみポリシー作成
            if *table == "battle_recruitments" {
                conn.execute_unprepared(&format!(
                    "CREATE POLICY {}_guild_policy ON worker.{} \
                     FOR ALL TO gbf_bot_guild \
                     USING (guild_id = current_setting('app.current_guild_id')::bigint)",
                    table, table
                ))
                .await?;
            } else if *table == "notifications" {
                // notifications は guild_id カラムで直接分離
                conn.execute_unprepared(&format!(
                    "CREATE POLICY {}_guild_policy ON worker.{} \
                     FOR ALL TO gbf_bot_guild \
                     USING (guild_id = current_setting('app.current_guild_id')::bigint) \
                     WITH CHECK (guild_id = current_setting('app.current_guild_id')::bigint)",
                    table, table
                ))
                .await?;
            } else if *table == "notification_rel_battle_recruitments" {
                conn.execute_unprepared(&format!(
                    "CREATE POLICY {}_guild_policy ON worker.{} \
                     FOR ALL TO gbf_bot_guild \
                     USING (EXISTS (\
                         SELECT 1 FROM worker.battle_recruitments br \
                         WHERE br.id = recruit_id \
                         AND br.guild_id = current_setting('app.current_guild_id')::bigint\
                     ))",
                    table, table
                ))
                .await?;
            } else if *table == "notification_rel_event_schedules" {
                // イベントスケジュール通知は全ギルドから参照可能（master データ由来）
                conn.execute_unprepared(&format!(
                    "CREATE POLICY {}_guild_policy ON worker.{} \
                     FOR ALL TO gbf_bot_guild \
                     USING (true)",
                    table, table
                ))
                .await?;
            } else if *table == "last_process_times" {
                // last_process_times は guild_id がなく、全ギルド共通
                conn.execute_unprepared(&format!(
                    "CREATE POLICY {}_guild_policy ON worker.{} \
                     FOR ALL TO gbf_bot_guild \
                     USING (true)",
                    table, table
                ))
                .await?;
            }
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // worker スキーマのポリシー削除と RLS 無効化
        let worker_tables = vec![
            "battle_recruitments",
            "notifications",
            "notification_rel_battle_recruitments",
            "notification_rel_event_schedules",
            "last_process_times",
        ];

        for table in &worker_tables {
            conn.execute_unprepared(&format!(
                "DROP POLICY IF EXISTS {}_guild_policy ON worker.{}",
                table, table
            ))
            .await?;

            conn.execute_unprepared(&format!(
                "ALTER TABLE worker.{} DISABLE ROW LEVEL SECURITY",
                table
            ))
            .await?;
        }

        // guild_master スキーマのポリシー削除と RLS 無効化
        let guild_master_tables = vec![
            "guilds",
            "guild_channels",
            "guild_spreadsheet_exports",
            "guild_spreadsheet_imports",
        ];

        for table in &guild_master_tables {
            conn.execute_unprepared(&format!(
                "DROP POLICY IF EXISTS {}_guild_policy ON guild_master.{}",
                table, table
            ))
            .await?;

            conn.execute_unprepared(&format!(
                "ALTER TABLE guild_master.{} DISABLE ROW LEVEL SECURITY",
                table
            ))
            .await?;
        }

        Ok(())
    }
}
