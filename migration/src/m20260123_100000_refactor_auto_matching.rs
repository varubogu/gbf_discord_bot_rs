//! 自動募集マッチング機能リファクタリング
//!
//! 変更内容:
//! - 新規テーブル: quest_matchings, quest_matching_users, auto_recruitment_quest_messages
//! - 変更テーブル: user_desired_quests に battle_style_id 追加、主キー変更
//! - 削除テーブル: matched_recruitment_channels, matched_recruitment_votes（投票機能廃止）
//! - 削除カラム: auto_recruitments.quest_message_id

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // ============================================
        // Phase 1: 新規テーブル作成
        // ============================================

        // quest_matchings テーブル作成（マッチング管理）
        conn.execute_unprepared(
            "CREATE TABLE worker.quest_matchings (
                guild_id BIGINT NOT NULL,
                id UUID NOT NULL DEFAULT gen_random_uuid(),
                quest_id INTEGER NOT NULL,
                scheduled_month INTEGER NOT NULL,
                scheduled_day INTEGER NOT NULL,
                scheduled_hour INTEGER NOT NULL,
                status VARCHAR(20) NOT NULL DEFAULT 'active',
                recruitment_id INTEGER,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                PRIMARY KEY (guild_id, id),
                CONSTRAINT fk_quest_matchings_quest FOREIGN KEY (quest_id)
                    REFERENCES master.quests(id) ON DELETE CASCADE,
                CONSTRAINT chk_quest_matchings_status CHECK (status IN ('active', 'completed', 'cancelled'))
            )",
        )
        .await?;

        // マッチング検索用インデックス
        conn.execute_unprepared(
            "CREATE INDEX idx_quest_matchings_schedule
            ON worker.quest_matchings(guild_id, quest_id, scheduled_month, scheduled_day, scheduled_hour)",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE INDEX idx_quest_matchings_status
            ON worker.quest_matchings(status)
            WHERE status = 'active'",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE TRIGGER set_quest_matchings_updated_at
            BEFORE UPDATE ON worker.quest_matchings
            FOR EACH ROW EXECUTE FUNCTION update_updated_at_column()",
        )
        .await?;

        // quest_matching_users テーブル作成（マッチング参加者管理）
        conn.execute_unprepared(
            "CREATE TABLE worker.quest_matching_users (
                guild_id BIGINT NOT NULL,
                matching_id UUID NOT NULL,
                user_id BIGINT NOT NULL,
                battle_style_id INTEGER,
                joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                left_at TIMESTAMPTZ,
                PRIMARY KEY (guild_id, matching_id, user_id),
                CONSTRAINT fk_matching_users_matching FOREIGN KEY (guild_id, matching_id)
                    REFERENCES worker.quest_matchings(guild_id, id) ON DELETE CASCADE
            )",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE INDEX idx_quest_matching_users_active
            ON worker.quest_matching_users(guild_id, matching_id)
            WHERE left_at IS NULL",
        )
        .await?;

        // auto_recruitment_quest_messages テーブル作成（クエストチャンネルのメッセージID管理）
        conn.execute_unprepared(
            "CREATE TABLE guild_master.auto_recruitment_quest_messages (
                guild_id BIGINT NOT NULL,
                quest_id INTEGER NOT NULL,
                message_id BIGINT NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                PRIMARY KEY (guild_id, quest_id),
                CONSTRAINT fk_quest_messages_guild FOREIGN KEY (guild_id)
                    REFERENCES guild_master.guilds(guild_id) ON DELETE CASCADE,
                CONSTRAINT fk_quest_messages_quest FOREIGN KEY (quest_id)
                    REFERENCES master.quests(id) ON DELETE CASCADE
            )",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE TRIGGER set_auto_recruitment_quest_messages_updated_at
            BEFORE UPDATE ON guild_master.auto_recruitment_quest_messages
            FOR EACH ROW EXECUTE FUNCTION update_updated_at_column()",
        )
        .await?;

        // ============================================
        // Phase 2: 既存テーブル変更
        // ============================================

        // user_desired_quests に battle_style_id カラム追加
        // まず既存データを一時テーブルに退避
        conn.execute_unprepared(
            "CREATE TEMP TABLE user_desired_quests_backup AS
            SELECT guild_id, user_id, quest_id, created_at, updated_at
            FROM guild_master.user_desired_quests",
        )
        .await?;

        // 既存テーブルを削除
        conn.execute_unprepared("DROP TABLE guild_master.user_desired_quests")
            .await?;

        // 新しいスキーマでテーブルを再作成
        conn.execute_unprepared(
            "CREATE TABLE guild_master.user_desired_quests (
                guild_id BIGINT NOT NULL,
                user_id BIGINT NOT NULL,
                quest_id INTEGER NOT NULL,
                battle_style_id INTEGER NOT NULL DEFAULT 0,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                PRIMARY KEY (guild_id, user_id, quest_id, battle_style_id),
                CONSTRAINT fk_user_quests_guild FOREIGN KEY (guild_id)
                    REFERENCES guild_master.guilds(guild_id) ON DELETE CASCADE,
                CONSTRAINT fk_user_quests_quest FOREIGN KEY (quest_id)
                    REFERENCES master.quests(id) ON DELETE CASCADE
            )",
        )
        .await?;

        // バックアップからデータを復元（battle_style_id = 0 として）
        conn.execute_unprepared(
            "INSERT INTO guild_master.user_desired_quests (guild_id, user_id, quest_id, battle_style_id, created_at, updated_at)
            SELECT guild_id, user_id, quest_id, 0, created_at, updated_at
            FROM user_desired_quests_backup",
        )
        .await?;

        // 一時テーブルを削除
        conn.execute_unprepared("DROP TABLE user_desired_quests_backup")
            .await?;

        // インデックス再作成
        conn.execute_unprepared(
            "CREATE INDEX idx_user_desired_quests_guild_user
            ON guild_master.user_desired_quests(guild_id, user_id)",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE INDEX idx_user_desired_quests_guild_quest
            ON guild_master.user_desired_quests(guild_id, quest_id)",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE TRIGGER set_user_desired_quests_updated_at
            BEFORE UPDATE ON guild_master.user_desired_quests
            FOR EACH ROW EXECUTE FUNCTION update_updated_at_column()",
        )
        .await?;

        // ============================================
        // Phase 2: 不要テーブル・カラム削除
        // ============================================

        // matched_recruitment_votes テーブル削除（投票機能廃止）
        conn.execute_unprepared("DROP TABLE IF EXISTS worker.matched_recruitment_votes")
            .await?;

        // matched_recruitment_channels テーブル削除（quest_matchingsに統合）
        conn.execute_unprepared("DROP TABLE IF EXISTS worker.matched_recruitment_channels")
            .await?;

        // auto_recruitments から quest_message_id カラムを削除
        // ※ カラムが存在する場合のみ削除
        conn.execute_unprepared(
            "ALTER TABLE guild_master.auto_recruitments
            DROP COLUMN IF EXISTS quest_message_id",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // auto_recruitments に quest_message_id カラムを追加
        conn.execute_unprepared(
            "ALTER TABLE guild_master.auto_recruitments
            ADD COLUMN IF NOT EXISTS quest_message_id BIGINT",
        )
        .await?;

        // matched_recruitment_channels テーブル再作成
        conn.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS worker.matched_recruitment_channels (
                id SERIAL PRIMARY KEY,
                guild_id BIGINT NOT NULL,
                channel_id BIGINT NOT NULL,
                message_id BIGINT NOT NULL,
                month INTEGER NOT NULL,
                day INTEGER NOT NULL,
                hour INTEGER NOT NULL,
                quest_id INTEGER,
                is_decided BOOLEAN NOT NULL DEFAULT FALSE,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                CONSTRAINT uq_matched_guild_datetime UNIQUE (guild_id, month, day, hour)
            )",
        )
        .await?;

        // matched_recruitment_votes テーブル再作成
        conn.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS worker.matched_recruitment_votes (
                id SERIAL PRIMARY KEY,
                matched_channel_id INTEGER NOT NULL,
                user_id BIGINT NOT NULL,
                quest_id INTEGER,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                CONSTRAINT fk_votes_matched FOREIGN KEY (matched_channel_id)
                    REFERENCES worker.matched_recruitment_channels(id) ON DELETE CASCADE,
                CONSTRAINT uq_votes_user UNIQUE (matched_channel_id, user_id)
            )",
        )
        .await?;

        // user_desired_quests を元のスキーマに戻す
        conn.execute_unprepared(
            "CREATE TEMP TABLE user_desired_quests_backup AS
            SELECT DISTINCT guild_id, user_id, quest_id, created_at, updated_at
            FROM guild_master.user_desired_quests",
        )
        .await?;

        conn.execute_unprepared("DROP TABLE guild_master.user_desired_quests")
            .await?;

        conn.execute_unprepared(
            "CREATE TABLE guild_master.user_desired_quests (
                guild_id BIGINT NOT NULL,
                user_id BIGINT NOT NULL,
                quest_id INTEGER NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                PRIMARY KEY (guild_id, user_id, quest_id),
                CONSTRAINT fk_user_quests_guild FOREIGN KEY (guild_id)
                    REFERENCES guild_master.guilds(guild_id) ON DELETE CASCADE,
                CONSTRAINT fk_user_quests_quest FOREIGN KEY (quest_id)
                    REFERENCES master.quests(id) ON DELETE CASCADE
            )",
        )
        .await?;

        conn.execute_unprepared(
            "INSERT INTO guild_master.user_desired_quests (guild_id, user_id, quest_id, created_at, updated_at)
            SELECT guild_id, user_id, quest_id, created_at, updated_at
            FROM user_desired_quests_backup",
        )
        .await?;

        conn.execute_unprepared("DROP TABLE user_desired_quests_backup")
            .await?;

        // 新規テーブル削除
        conn.execute_unprepared("DROP TABLE IF EXISTS guild_master.auto_recruitment_quest_messages")
            .await?;

        conn.execute_unprepared("DROP TABLE IF EXISTS worker.quest_matching_users")
            .await?;

        conn.execute_unprepared("DROP TABLE IF EXISTS worker.quest_matchings")
            .await?;

        Ok(())
    }
}
