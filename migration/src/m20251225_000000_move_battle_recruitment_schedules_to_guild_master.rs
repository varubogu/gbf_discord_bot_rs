use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // battle_recruitment_schedules を worker から guild_master へ移動
        conn.execute_unprepared(
            "ALTER TABLE worker.battle_recruitment_schedules SET SCHEMA guild_master",
        )
        .await?;

        // battle_recruitment_schedule_days を worker から guild_master へ移動
        conn.execute_unprepared(
            "ALTER TABLE worker.battle_recruitment_schedule_days SET SCHEMA guild_master",
        )
        .await?;

        // guild_master スキーマのテーブルに RLS を有効化
        conn.execute_unprepared(
            "ALTER TABLE guild_master.battle_recruitment_schedules ENABLE ROW LEVEL SECURITY",
        )
        .await?;

        // battle_recruitment_schedules テーブルに guild_id によるポリシーを設定
        conn.execute_unprepared(
            "CREATE POLICY battle_recruitment_schedules_guild_policy ON guild_master.battle_recruitment_schedules \
             FOR ALL TO gbf_bot_guild \
             USING (guild_id = current_setting('app.current_guild_id')::bigint) \
             WITH CHECK (guild_id = current_setting('app.current_guild_id')::bigint)",
        )
        .await?;

        // battle_recruitment_schedule_days テーブルに RLS を有効化
        conn.execute_unprepared(
            "ALTER TABLE guild_master.battle_recruitment_schedule_days ENABLE ROW LEVEL SECURITY",
        )
        .await?;

        // battle_recruitment_schedule_days テーブルに schedule_id 経由でポリシーを設定
        conn.execute_unprepared(
            "CREATE POLICY battle_recruitment_schedule_days_guild_policy ON guild_master.battle_recruitment_schedule_days \
             FOR ALL TO gbf_bot_guild \
             USING (EXISTS (\
                 SELECT 1 FROM guild_master.battle_recruitment_schedules brs \
                 WHERE brs.id = schedule_id \
                 AND brs.guild_id = current_setting('app.current_guild_id')::bigint\
             )) \
             WITH CHECK (EXISTS (\
                 SELECT 1 FROM guild_master.battle_recruitment_schedules brs \
                 WHERE brs.id = schedule_id \
                 AND brs.guild_id = current_setting('app.current_guild_id')::bigint\
             ))",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // battle_recruitment_schedule_days テーブルのポリシーを削除
        conn.execute_unprepared(
            "DROP POLICY IF EXISTS battle_recruitment_schedule_days_guild_policy ON guild_master.battle_recruitment_schedule_days",
        )
        .await?;

        // battle_recruitment_schedule_days テーブルの RLS を無効化
        conn.execute_unprepared(
            "ALTER TABLE guild_master.battle_recruitment_schedule_days DISABLE ROW LEVEL SECURITY",
        )
        .await?;

        // battle_recruitment_schedules テーブルのポリシーを削除
        conn.execute_unprepared(
            "DROP POLICY IF EXISTS battle_recruitment_schedules_guild_policy ON guild_master.battle_recruitment_schedules",
        )
        .await?;

        // battle_recruitment_schedules テーブルの RLS を無効化
        conn.execute_unprepared(
            "ALTER TABLE guild_master.battle_recruitment_schedules DISABLE ROW LEVEL SECURITY",
        )
        .await?;

        // テーブルを元のスキーマに戻す
        conn.execute_unprepared(
            "ALTER TABLE guild_master.battle_recruitment_schedules SET SCHEMA worker",
        )
        .await?;

        conn.execute_unprepared(
            "ALTER TABLE guild_master.battle_recruitment_schedule_days SET SCHEMA worker",
        )
        .await?;

        Ok(())
    }
}
