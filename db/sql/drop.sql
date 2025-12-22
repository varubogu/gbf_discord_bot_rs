-- 開発用データベースをクリアするSQL
-- 使用方法: psql -h <host> -U postgres -d postgres -f db/drop.sql

-- 既存の接続を強制終了
SELECT pg_terminate_backend(pid)
FROM pg_stat_activity
WHERE datname = 'gbf_bot_db'
  AND pid <> pg_backend_pid();

-- データベースを削除
DROP DATABASE IF EXISTS gbf_bot_db;

-- ユーザーも削除する場合（必要に応じてコメント解除）
-- DROP USER IF EXISTS gbf_bot_user;
-- DROP USER IF EXISTS migration_user;
