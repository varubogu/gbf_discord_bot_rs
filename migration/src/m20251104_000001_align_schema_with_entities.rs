use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{DatabaseBackend, Statement};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // UUID生成・暗号化関数を利用するためにpgcrypto拡張を有効化
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"CREATE EXTENSION IF NOT EXISTS "pgcrypto";"#,
        ))
        .await?;

        // environments: サロゲートキーを廃止し、keyを主キーに変更
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"
DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_name = 'environments'
          AND column_name = 'id'
    ) THEN
        ALTER TABLE environments DROP CONSTRAINT IF EXISTS environments_pkey;
        ALTER TABLE environments DROP COLUMN id;
        DROP SEQUENCE IF EXISTS environments_id_seq;
    END IF;
END $$;
"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"UPDATE environments SET created_at = NOW() WHERE created_at IS NULL;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"UPDATE environments SET updated_at = NOW() WHERE updated_at IS NULL;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE environments ALTER COLUMN key SET NOT NULL;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE environments ALTER COLUMN value SET NOT NULL;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE environments ALTER COLUMN created_at SET NOT NULL;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE environments ALTER COLUMN updated_at SET NOT NULL;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'environments_pkey'
          AND conrelid = 'environments'::regclass
    ) THEN
        ALTER TABLE environments
            ADD CONSTRAINT environments_pkey PRIMARY KEY (key);
    END IF;
END $$;
"#,
        ))
        .await?;

        // message_texts: message_text_idを主キーに昇格し、guild_idを廃止
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE message_texts DROP CONSTRAINT IF EXISTS message_texts_pkey;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE message_texts RENAME COLUMN id TO id_old;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE message_texts RENAME COLUMN message_text_id TO id;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE message_texts DROP COLUMN IF EXISTS guild_id;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE message_texts ALTER COLUMN id SET NOT NULL;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE message_texts ALTER COLUMN message_jp SET NOT NULL;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"UPDATE message_texts SET created_at = NOW() WHERE created_at IS NULL;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"UPDATE message_texts SET updated_at = NOW() WHERE updated_at IS NULL;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE message_texts ALTER COLUMN created_at SET NOT NULL;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE message_texts ALTER COLUMN updated_at SET NOT NULL;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE message_texts ADD CONSTRAINT message_texts_pkey PRIMARY KEY (id);"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE message_texts DROP COLUMN id_old;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"DROP SEQUENCE IF EXISTS message_texts_id_seq;"#,
        ))
        .await?;

        // battle_types: display_nameへの名称統一と不足列の補完
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"
DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_name = 'battle_types'
          AND column_name = 'type_id'
    ) THEN
        ALTER TABLE battle_types RENAME COLUMN type_id TO id;
    END IF;
END $$;
"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"
DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_name = 'battle_types'
          AND column_name = 'name'
    ) THEN
        ALTER TABLE battle_types RENAME COLUMN name TO display_name;
    END IF;
END $$;
"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE battle_types ADD COLUMN IF NOT EXISTS reactions TEXT;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE battle_types ADD COLUMN IF NOT EXISTS sort_order INTEGER NOT NULL DEFAULT 0;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE battle_types ADD COLUMN IF NOT EXISTS created_at TIMESTAMPTZ DEFAULT NOW();"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE battle_types ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ DEFAULT NOW();"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"UPDATE battle_types SET created_at = NOW() WHERE created_at IS NULL;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"UPDATE battle_types SET updated_at = NOW() WHERE updated_at IS NULL;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE battle_types ALTER COLUMN id SET NOT NULL;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE battle_types ALTER COLUMN display_name SET NOT NULL;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE battle_types ALTER COLUMN created_at SET NOT NULL;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE battle_types ALTER COLUMN updated_at SET NOT NULL;"#,
        ))
        .await?;

        // elements: カラム名称と不足列を整備
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"
DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_name = 'elements'
          AND column_name = 'name'
    ) THEN
        ALTER TABLE elements RENAME COLUMN name TO name_jp;
    END IF;
END $$;
"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE elements ADD COLUMN IF NOT EXISTS reaction_stamp TEXT;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE elements ADD COLUMN IF NOT EXISTS name_en TEXT;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE elements ALTER COLUMN id SET NOT NULL;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE elements ALTER COLUMN name_jp SET NOT NULL;"#,
        ))
        .await?;

        // quests: available_battle_stylesの型をTEXTへ
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"
ALTER TABLE quests
    ALTER COLUMN available_battle_styles TYPE TEXT
    USING available_battle_styles::TEXT;
"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"UPDATE quests SET available_battle_styles = '' WHERE available_battle_styles IS NULL;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE quests ALTER COLUMN available_battle_styles SET NOT NULL;"#,
        ))
        .await?;

        // quest_aliases: alias_kana_smallを追加
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE quest_aliases ADD COLUMN IF NOT EXISTS alias_kana_small TEXT;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"UPDATE quest_aliases SET alias_kana_small = '' WHERE alias_kana_small IS NULL;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE quest_aliases ALTER COLUMN alias_kana_small SET NOT NULL;"#,
        ))
        .await?;

        // guilds: カラム名称の正規化と不要列の削除
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"
DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_name = 'guilds'
          AND column_name = 'id'
    ) THEN
        ALTER TABLE guilds RENAME COLUMN id TO guild_id;
    END IF;
END $$;
"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"
ALTER TABLE guilds
    ALTER COLUMN guild_id TYPE BIGINT;
"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"
DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_name = 'guilds'
          AND column_name = 'guild_name'
    ) THEN
        ALTER TABLE guilds RENAME COLUMN guild_name TO name;
    END IF;
END $$;
"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE guilds DROP COLUMN IF EXISTS discord_guild_id;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE guilds DROP COLUMN IF EXISTS notification_channel_id;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE guilds ADD COLUMN IF NOT EXISTS recruit_channel_id BIGINT;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE guilds ADD COLUMN IF NOT EXISTS timezone TEXT;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE guilds ADD COLUMN IF NOT EXISTS default_recruit_duration INTEGER;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE guilds ADD COLUMN IF NOT EXISTS created_at TIMESTAMPTZ DEFAULT NOW();"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE guilds ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ DEFAULT NOW();"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"UPDATE guilds SET created_at = NOW() WHERE created_at IS NULL;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"UPDATE guilds SET updated_at = NOW() WHERE updated_at IS NULL;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE guilds ALTER COLUMN guild_id SET NOT NULL;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE guilds ALTER COLUMN name SET NOT NULL;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE guilds ALTER COLUMN created_at SET NOT NULL;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE guilds ALTER COLUMN updated_at SET NOT NULL;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE guilds DROP CONSTRAINT IF EXISTS guilds_pkey;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE guilds ADD CONSTRAINT guilds_pkey PRIMARY KEY (guild_id);"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"DROP SEQUENCE IF EXISTS guilds_id_seq;"#,
        ))
        .await?;

        // event_schedule_details / event_schedules: 主キーをUUID化し、通知関連列を整備
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE event_schedules ADD COLUMN IF NOT EXISTS id_uuid UUID DEFAULT gen_random_uuid();"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE event_schedule_details ADD COLUMN IF NOT EXISTS id_uuid UUID DEFAULT gen_random_uuid();"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE notification_rel_event_schedules ADD COLUMN IF NOT EXISTS event_schedule_id_uuid UUID;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE notification_rel_event_schedules ADD COLUMN IF NOT EXISTS event_schedule_detail_id_uuid UUID;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"
DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_name = 'notification_rel_event_schedules'
          AND column_name = 'event_schedule_id'
    ) THEN
        UPDATE notification_rel_event_schedules AS n
        SET event_schedule_id_uuid = e.id_uuid
        FROM event_schedules AS e
        WHERE n.event_schedule_id = e.id;
    END IF;
END $$;
"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"
DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_name = 'notification_rel_event_schedules'
          AND column_name = 'event_schedule_detail_id'
    ) THEN
        UPDATE notification_rel_event_schedules AS n
        SET event_schedule_detail_id_uuid = d.id_uuid
        FROM event_schedule_details AS d
        WHERE n.event_schedule_detail_id = d.id;
    END IF;
END $$;
"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE notification_rel_event_schedules DROP CONSTRAINT IF EXISTS fk_notification_rel_event_schedules_event_schedule_id;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE notification_rel_event_schedules DROP CONSTRAINT IF EXISTS fk_notification_rel_event_schedules_event_schedule_detail_id;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE notification_rel_event_schedules DROP CONSTRAINT IF EXISTS pk_notification_rel_event_schedules;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE notification_rel_event_schedules DROP COLUMN IF EXISTS event_schedule_id;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE notification_rel_event_schedules DROP COLUMN IF EXISTS event_schedule_detail_id;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"
ALTER TABLE event_schedules DROP CONSTRAINT IF EXISTS event_schedules_pkey;
"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"
ALTER TABLE event_schedule_details DROP CONSTRAINT IF EXISTS event_schedule_details_pkey;
"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE event_schedules RENAME COLUMN id TO id_old;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE event_schedule_details RENAME COLUMN id TO id_old;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE event_schedules RENAME COLUMN id_uuid TO id;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE event_schedule_details RENAME COLUMN id_uuid TO id;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE event_schedules ALTER COLUMN id SET NOT NULL;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE event_schedule_details ALTER COLUMN id SET NOT NULL;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE event_schedules ADD CONSTRAINT event_schedules_pkey PRIMARY KEY (id);"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE event_schedule_details ADD CONSTRAINT event_schedule_details_pkey PRIMARY KEY (id);"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE event_schedules DROP COLUMN id_old;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE event_schedule_details DROP COLUMN id_old;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"DROP SEQUENCE IF EXISTS event_schedules_id_seq;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"DROP SEQUENCE IF EXISTS event_schedule_details_id_seq;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"
DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_name = 'notification_rel_event_schedules'
          AND column_name = 'event_schedule_id_uuid'
    ) THEN
        ALTER TABLE notification_rel_event_schedules
            RENAME COLUMN event_schedule_id_uuid TO event_schedule_id;
    END IF;
END $$;
"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"
DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_name = 'notification_rel_event_schedules'
          AND column_name = 'event_schedule_detail_id_uuid'
    ) THEN
        ALTER TABLE notification_rel_event_schedules
            RENAME COLUMN event_schedule_detail_id_uuid TO event_schedule_detail_id;
    END IF;
END $$;
"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE notification_rel_event_schedules ALTER COLUMN event_schedule_id SET NOT NULL;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE notification_rel_event_schedules ADD CONSTRAINT pk_notification_rel_event_schedules PRIMARY KEY (event_schedule_id, notification_id);"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE notification_rel_event_schedules ADD CONSTRAINT fk_notification_rel_event_schedules_event_schedule_id FOREIGN KEY (event_schedule_id) REFERENCES event_schedules (id) ON DELETE CASCADE ON UPDATE CASCADE;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE notification_rel_event_schedules ADD CONSTRAINT fk_notification_rel_event_schedules_event_schedule_detail_id FOREIGN KEY (event_schedule_detail_id) REFERENCES event_schedule_details (id) ON DELETE SET NULL ON UPDATE CASCADE;"#,
        ))
        .await?;

        // event_schedule_details: 追加列の整備
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE event_schedule_details ADD COLUMN IF NOT EXISTS notification_channel_type INTEGER;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"UPDATE event_schedule_details SET notification_channel_type = 0 WHERE notification_channel_type IS NULL;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE event_schedule_details ALTER COLUMN notification_channel_type SET NOT NULL;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE event_schedule_details DROP COLUMN IF EXISTS guild_id;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE event_schedule_details DROP COLUMN IF EXISTS channel_id;"#,
        ))
        .await?;

        // reactions列はNULL禁止、空文字は許容（TEXT NOT NULLを維持）
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"UPDATE event_schedule_details SET reactions = '' WHERE reactions IS NULL;"#,
        ))
        .await?;

        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            r#"ALTER TABLE event_schedule_details ALTER COLUMN reactions SET NOT NULL;"#,
        ))
        .await?;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Custom(
            "m20251104_000001_align_schema_with_entities cannot be reverted automatically"
                .to_owned(),
        ))
    }
}
