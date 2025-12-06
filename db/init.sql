-- ================================
-- GBF Discord Bot データベース初期化スクリプト
-- ================================
-- このスクリプトはDockerコンテナ起動時に実行され、
-- データベースロールとデータベースを作成します。
-- マイグレーションはアプリケーション側で実行されます。
-- ================================

-- ロール作成（既存の場合はスキップ）
-- パスワードは本番環境では必ず変更してください

-- 1. System ロール（スケジューラー、バックグラウンドタスク用）
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'gbf_bot_system') THEN
        CREATE ROLE gbf_bot_system WITH LOGIN PASSWORD 'change_this_system_password';
    END IF;
END
$$;

-- System ロールはRLSをバイパス（全ギルドのデータにアクセス可能）
ALTER ROLE gbf_bot_system WITH BYPASSRLS;

-- 2. Guild ロール（通常のDiscordコマンド実行用）
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'gbf_bot_guild') THEN
        CREATE ROLE gbf_bot_guild WITH LOGIN PASSWORD 'change_this_guild_password';
    END IF;
END
$$;

-- Guild ロールはRLSポリシーに従う（BYPASSRLSなし）
-- app.current_guild_id でギルドごとにデータを分離

-- 3. Global ロール（マスターデータ更新、スプレッドシート同期用）
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'gbf_bot_global') THEN
        CREATE ROLE gbf_bot_global WITH LOGIN PASSWORD 'change_this_global_password';
    END IF;
END
$$;

-- Global ロールはRLSをバイパス（マスターデータ更新のため）
ALTER ROLE gbf_bot_global WITH BYPASSRLS;

-- 4. Admin ロール（マイグレーション実行、管理操作用）
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'gbf_bot_admin') THEN
        CREATE ROLE gbf_bot_admin WITH LOGIN PASSWORD 'change_this_admin_password';
    END IF;
END
$$;

-- Admin ロールはRLSをバイパス＋スキーマ作成権限
ALTER ROLE gbf_bot_admin WITH BYPASSRLS CREATEDB;

-- データベース作成（既存の場合はスキップ）
SELECT 'CREATE DATABASE gbf_bot_db WITH OWNER gbf_bot_admin ENCODING ''UTF8'''
WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = 'gbf_bot_db')\gexec

-- データベースへの接続権限付与
\connect gbf_bot_db

GRANT CONNECT ON DATABASE gbf_bot_db TO gbf_bot_system;
GRANT CONNECT ON DATABASE gbf_bot_db TO gbf_bot_guild;
GRANT CONNECT ON DATABASE gbf_bot_db TO gbf_bot_global;
GRANT CONNECT ON DATABASE gbf_bot_db TO gbf_bot_admin;

-- スキーマ作成権限（adminロールのみ）
-- マイグレーション実行時に必要
GRANT CREATE ON DATABASE gbf_bot_db TO gbf_bot_admin;

-- 完了メッセージ
DO $$
BEGIN
    RAISE NOTICE '========================================';
    RAISE NOTICE 'GBF Discord Bot データベース初期化完了';
    RAISE NOTICE '========================================';
    RAISE NOTICE 'データベース: gbf_bot_db';
    RAISE NOTICE 'ロール:';
    RAISE NOTICE '  - gbf_bot_system (BYPASSRLS)';
    RAISE NOTICE '  - gbf_bot_guild (RLS適用)';
    RAISE NOTICE '  - gbf_bot_global (BYPASSRLS)';
    RAISE NOTICE '  - gbf_bot_admin (BYPASSRLS, CREATEDB)';
    RAISE NOTICE '';
    RAISE NOTICE '次のステップ:';
    RAISE NOTICE '  1. .envファイルでパスワードを設定';
    RAISE NOTICE '  2. マイグレーション実行: cargo run -- migrate';
    RAISE NOTICE '========================================';
END
$$;
