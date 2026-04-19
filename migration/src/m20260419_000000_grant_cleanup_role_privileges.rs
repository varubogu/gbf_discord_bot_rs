use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // cleanupロールが存在する環境では、クリーンアップ実行に必要な権限を付与する
        conn.execute_unprepared(
            "DO $$
             BEGIN
                 IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'gbf_bot_cleanup') THEN
                     EXECUTE format(
                         'GRANT CONNECT ON DATABASE %I TO gbf_bot_cleanup',
                         current_database()
                     );
                     GRANT USAGE ON SCHEMA worker TO gbf_bot_cleanup;
                     GRANT DELETE, SELECT ON worker.battle_recruitments TO gbf_bot_cleanup;
                     GRANT DELETE, SELECT ON worker.notifications TO gbf_bot_cleanup;
                     GRANT DELETE, SELECT ON worker.scheduled_tasks TO gbf_bot_cleanup;
                 ELSE
                     RAISE NOTICE 'gbf_bot_cleanup ロールが存在しないため、権限付与をスキップします';
                 END IF;
             END
             $$;",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        conn.execute_unprepared(
            "DO $$
             BEGIN
                 IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'gbf_bot_cleanup') THEN
                     REVOKE DELETE, SELECT ON worker.battle_recruitments FROM gbf_bot_cleanup;
                     REVOKE DELETE, SELECT ON worker.notifications FROM gbf_bot_cleanup;
                     REVOKE DELETE, SELECT ON worker.scheduled_tasks FROM gbf_bot_cleanup;
                     REVOKE USAGE ON SCHEMA worker FROM gbf_bot_cleanup;
                     EXECUTE format(
                         'REVOKE CONNECT ON DATABASE %I FROM gbf_bot_cleanup',
                         current_database()
                     );
                 END IF;
             END
             $$;",
        )
        .await?;

        Ok(())
    }
}
