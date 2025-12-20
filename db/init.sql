-- ================================
-- GBF Discord Bot データベース初期化スクリプト
-- ================================
-- このスクリプトはシェルスクリプトから呼び出され、
-- データベースロールとデータベースを作成します。
-- マイグレーションはアプリケーション側で実行されます。
--
-- 使用方法:
--   psql -v system_password="..." -v guild_password="..." \
--        -v global_password="..." -v admin_password="..." \
--        -v db_name="gbf_bot_db" -f init.sql
-- ================================

-- ロール作成（既存の場合はスキップ）

-- 1. System ロール（スケジューラー、バックグラウンドタスク用）
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'gbf_bot_system') THEN
        CREATE ROLE gbf_bot_system WITH LOGIN PASSWORD :'system_password';
    ELSE
        ALTER ROLE gbf_bot_system WITH PASSWORD :'system_password';
    END IF;
END
$$;

-- System ロールはRLSをバイパス（全ギルドのデータにアクセス可能）
ALTER ROLE gbf_bot_system WITH BYPASSRLS;

-- 2. Guild ロール（通常のDiscordコマンド実行用）
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'gbf_bot_guild') THEN
        CREATE ROLE gbf_bot_guild WITH LOGIN PASSWORD :'guild_password';
    ELSE
        ALTER ROLE gbf_bot_guild WITH PASSWORD :'guild_password';
    END IF;
END
$$;

-- Guild ロールはRLSポリシーに従う（BYPASSRLSなし）
-- app.current_guild_id でギルドごとにデータを分離

-- 3. Global ロール（マスターデータ更新、スプレッドシート同期用）
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'gbf_bot_global') THEN
        CREATE ROLE gbf_bot_global WITH LOGIN PASSWORD :'global_password';
    ELSE
        ALTER ROLE gbf_bot_global WITH PASSWORD :'global_password';
    END IF;
END
$$;

-- Global ロールはRLSをバイパス（マスターデータ更新のため）
ALTER ROLE gbf_bot_global WITH BYPASSRLS;

-- 4. Admin ロール（マイグレーション実行、管理操作用）
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'gbf_bot_admin') THEN
        CREATE ROLE gbf_bot_admin WITH LOGIN PASSWORD :'admin_password';
    ELSE
        ALTER ROLE gbf_bot_admin WITH PASSWORD :'admin_password';
    END IF;
END
$$;

-- Admin ロールはRLSをバイパス＋スキーマ作成権限
ALTER ROLE gbf_bot_admin WITH BYPASSRLS CREATEDB;

-- データベース作成（既存の場合はスキップ）
SELECT format('CREATE DATABASE %I WITH OWNER gbf_bot_admin ENCODING ''UTF8''', :'db_name')
WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = :'db_name')\gexec

-- データベースへの接続権限付与
\connect :db_name

GRANT CONNECT ON DATABASE :db_name TO gbf_bot_system;
GRANT CONNECT ON DATABASE :db_name TO gbf_bot_guild;
GRANT CONNECT ON DATABASE :db_name TO gbf_bot_global;
GRANT CONNECT ON DATABASE :db_name TO gbf_bot_admin;

-- スキーマ作成権限（adminロールのみ）
-- マイグレーション実行時に必要
GRANT CREATE ON DATABASE :db_name TO gbf_bot_admin;

-- 完了メッセージ
DO $$
BEGIN
    RAISE NOTICE '========================================';
    RAISE NOTICE 'GBF Discord Bot データベース初期化完了';
    RAISE NOTICE '========================================';
    RAISE NOTICE 'ロール:';
    RAISE NOTICE '  - gbf_bot_system (BYPASSRLS)';
    RAISE NOTICE '  - gbf_bot_guild (RLS適用)';
    RAISE NOTICE '  - gbf_bot_global (BYPASSRLS)';
    RAISE NOTICE '  - gbf_bot_admin (BYPASSRLS, CREATEDB)';
    RAISE NOTICE '';
    RAISE NOTICE '次のステップ:';
    RAISE NOTICE '  1. マイグレーション実行: cargo run -- migrate';
    RAISE NOTICE '========================================';
END
$$;
