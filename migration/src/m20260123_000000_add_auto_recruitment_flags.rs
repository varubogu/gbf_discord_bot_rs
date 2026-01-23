//! 自動募集テーブルにBot作成フラグとメッセージIDを追加するマイグレーション
//!
//! - auto_recruitments: マッチング/クエストチャンネルのBot作成フラグとメッセージID
//! - auto_recruitment_channels: 日時チャンネルのBot作成フラグとメッセージID

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // auto_recruitments テーブルにカラム追加
        conn.execute_unprepared(
            "ALTER TABLE guild_master.auto_recruitments
            ADD COLUMN matching_channel_is_bot_created BOOLEAN NOT NULL DEFAULT FALSE,
            ADD COLUMN quest_channel_is_bot_created BOOLEAN NOT NULL DEFAULT FALSE,
            ADD COLUMN matching_message_id BIGINT,
            ADD COLUMN quest_message_id BIGINT",
        )
        .await?;

        // auto_recruitment_channels テーブルにカラム追加
        conn.execute_unprepared(
            "ALTER TABLE guild_master.auto_recruitment_channels
            ADD COLUMN is_bot_created BOOLEAN NOT NULL DEFAULT FALSE,
            ADD COLUMN message_id BIGINT",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // auto_recruitment_channels テーブルからカラム削除
        conn.execute_unprepared(
            "ALTER TABLE guild_master.auto_recruitment_channels
            DROP COLUMN IF EXISTS is_bot_created,
            DROP COLUMN IF EXISTS message_id",
        )
        .await?;

        // auto_recruitments テーブルからカラム削除
        conn.execute_unprepared(
            "ALTER TABLE guild_master.auto_recruitments
            DROP COLUMN IF EXISTS matching_channel_is_bot_created,
            DROP COLUMN IF EXISTS quest_channel_is_bot_created,
            DROP COLUMN IF EXISTS matching_message_id,
            DROP COLUMN IF EXISTS quest_message_id",
        )
        .await?;

        Ok(())
    }
}
