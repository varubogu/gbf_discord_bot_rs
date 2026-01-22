use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // ============================================
        // guild_master スキーマのテーブル
        // ============================================

        // auto_recruitments テーブル作成（ギルド自動募集設定）
        conn.execute_unprepared(
            "CREATE TABLE guild_master.auto_recruitments (
                guild_id BIGINT PRIMARY KEY NOT NULL,
                category_id BIGINT NOT NULL,
                matching_channel_id BIGINT,
                quest_channel_id BIGINT,
                days_range INTEGER NOT NULL DEFAULT 7,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                CONSTRAINT fk_auto_recruitments_guild FOREIGN KEY (guild_id)
                    REFERENCES guild_master.guilds(guild_id) ON DELETE CASCADE
            )",
        )
        .await?;

        // updated_at トリガー追加
        conn.execute_unprepared(
            "CREATE TRIGGER set_auto_recruitments_updated_at
            BEFORE UPDATE ON guild_master.auto_recruitments
            FOR EACH ROW EXECUTE FUNCTION set_updated_at()",
        )
        .await?;

        // auto_recruitment_channels テーブル作成（日時チャンネル管理）
        conn.execute_unprepared(
            "CREATE TABLE guild_master.auto_recruitment_channels (
                id SERIAL PRIMARY KEY,
                guild_id BIGINT NOT NULL,
                channel_id BIGINT NOT NULL,
                month INTEGER NOT NULL,
                day INTEGER NOT NULL,
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                CONSTRAINT fk_auto_channels_guild FOREIGN KEY (guild_id)
                    REFERENCES guild_master.guilds(guild_id) ON DELETE CASCADE,
                CONSTRAINT uq_auto_channels_guild_channel UNIQUE (guild_id, channel_id)
            )",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE INDEX idx_auto_recruitment_channels_guild
            ON guild_master.auto_recruitment_channels(guild_id)",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE TRIGGER set_auto_recruitment_channels_updated_at
            BEFORE UPDATE ON guild_master.auto_recruitment_channels
            FOR EACH ROW EXECUTE FUNCTION set_updated_at()",
        )
        .await?;

        // user_desired_quests テーブル作成（ユーザー希望クエスト）
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
            FOR EACH ROW EXECUTE FUNCTION set_updated_at()",
        )
        .await?;

        // auto_recruitment_participants テーブル作成（参加可能時間）
        conn.execute_unprepared(
            "CREATE TABLE guild_master.auto_recruitment_participants (
                guild_id BIGINT NOT NULL,
                user_id BIGINT NOT NULL,
                month INTEGER NOT NULL,
                day INTEGER NOT NULL,
                hour INTEGER NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                PRIMARY KEY (guild_id, user_id, month, day, hour),
                CONSTRAINT fk_auto_participants_guild FOREIGN KEY (guild_id)
                    REFERENCES guild_master.guilds(guild_id) ON DELETE CASCADE
            )",
        )
        .await?;

        // マッチング用のインデックス（同じ日時の参加者を効率的に検索）
        conn.execute_unprepared(
            "CREATE INDEX idx_auto_recruitment_participants_matching
            ON guild_master.auto_recruitment_participants(guild_id, month, day, hour)",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE TRIGGER set_auto_recruitment_participants_updated_at
            BEFORE UPDATE ON guild_master.auto_recruitment_participants
            FOR EACH ROW EXECUTE FUNCTION set_updated_at()",
        )
        .await?;

        // ============================================
        // worker スキーマのテーブル
        // ============================================

        // matched_recruitment_channels テーブル作成（マッチング済み募集）
        conn.execute_unprepared(
            "CREATE TABLE worker.matched_recruitment_channels (
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

        conn.execute_unprepared(
            "CREATE INDEX idx_matched_recruitment_channels_guild
            ON worker.matched_recruitment_channels(guild_id)",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE INDEX idx_matched_recruitment_channels_undecided
            ON worker.matched_recruitment_channels(is_decided)
            WHERE is_decided = false",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE TRIGGER set_matched_recruitment_channels_updated_at
            BEFORE UPDATE ON worker.matched_recruitment_channels
            FOR EACH ROW EXECUTE FUNCTION set_updated_at()",
        )
        .await?;

        // matched_recruitment_votes テーブル作成（マッチング投票）
        conn.execute_unprepared(
            "CREATE TABLE worker.matched_recruitment_votes (
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

        conn.execute_unprepared(
            "CREATE INDEX idx_matched_recruitment_votes_matched
            ON worker.matched_recruitment_votes(matched_channel_id)",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE TRIGGER set_matched_recruitment_votes_updated_at
            BEFORE UPDATE ON worker.matched_recruitment_votes
            FOR EACH ROW EXECUTE FUNCTION set_updated_at()",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // worker スキーマのテーブル削除
        conn.execute_unprepared("DROP TABLE IF EXISTS worker.matched_recruitment_votes")
            .await?;

        conn.execute_unprepared("DROP TABLE IF EXISTS worker.matched_recruitment_channels")
            .await?;

        // guild_master スキーマのテーブル削除
        conn.execute_unprepared("DROP TABLE IF EXISTS guild_master.auto_recruitment_participants")
            .await?;

        conn.execute_unprepared("DROP TABLE IF EXISTS guild_master.user_desired_quests")
            .await?;

        conn.execute_unprepared("DROP TABLE IF EXISTS guild_master.auto_recruitment_channels")
            .await?;

        conn.execute_unprepared("DROP TABLE IF EXISTS guild_master.auto_recruitments")
            .await?;

        Ok(())
    }
}
