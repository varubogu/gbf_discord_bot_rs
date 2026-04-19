-- ================================
-- Cleanupロール作成スクリプト（既存環境用）
-- ================================
-- データクリーンアップ専用のデータベースロールを作成し、権限を付与します。
-- 既存環境にCleanupロールを追加する場合に使用してください。
--
-- 前提条件:
--   - データベースとworkerスキーマが既に存在すること
--   - マイグレーションが実行済みであること
--
-- 使用方法:
--   psql -v cleanup_password="your_cleanup_password" \
--        -d gbf_bot_db -f create_cleanup_role.sql
--
-- または環境変数から:
--   export CLEANUP_DB_PASSWORD="your_cleanup_password"
--   psql -v cleanup_password="$CLEANUP_DB_PASSWORD" \
--        -d gbf_bot_db -f create_cleanup_role.sql
-- ================================

-- Cleanupロールを作成（既に存在する場合はスキップ）
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'gbf_bot_cleanup') THEN
        CREATE ROLE gbf_bot_cleanup WITH LOGIN PASSWORD :'cleanup_password';
        RAISE NOTICE 'gbf_bot_cleanup ロールを作成しました';
    ELSE
        RAISE NOTICE 'gbf_bot_cleanup ロールは既に存在します';
        -- 既存の場合もパスワードを更新
        EXECUTE format('ALTER ROLE gbf_bot_cleanup WITH LOGIN PASSWORD %L', :'cleanup_password');
        RAISE NOTICE 'gbf_bot_cleanup ロールのパスワードを更新しました';
    END IF;
END
$$;

-- RLSをバイパス（全ギルドのデータを対象とするため）
ALTER ROLE gbf_bot_cleanup WITH BYPASSRLS;

-- データベース接続権限を付与（接続中DBに対して実行）
DO $$
BEGIN
    EXECUTE format(
        'GRANT CONNECT ON DATABASE %I TO gbf_bot_cleanup',
        current_database()
    );
END
$$;

-- workerスキーマへの接続権限
GRANT USAGE ON SCHEMA worker TO gbf_bot_cleanup;

-- 削除対象テーブルにDELETE + SELECT権限を付与
GRANT DELETE, SELECT ON worker.battle_recruitments TO gbf_bot_cleanup;
GRANT DELETE, SELECT ON worker.notifications TO gbf_bot_cleanup;
GRANT DELETE, SELECT ON worker.scheduled_tasks TO gbf_bot_cleanup;

-- コメント追加
COMMENT ON ROLE gbf_bot_cleanup IS 'データクリーンアップ専用ロール - workerスキーマの特定テーブルに対するDELETE + SELECT権限のみ';

-- 完了メッセージ
DO $$
BEGIN
    RAISE NOTICE '========================================';
    RAISE NOTICE 'Cleanupロール作成・権限付与完了';
    RAISE NOTICE '========================================';
    RAISE NOTICE 'ロール: gbf_bot_cleanup';
    RAISE NOTICE '権限:';
    RAISE NOTICE '  - worker.battle_recruitments: DELETE, SELECT';
    RAISE NOTICE '  - worker.notifications: DELETE, SELECT';
    RAISE NOTICE '  - worker.scheduled_tasks: DELETE, SELECT';
    RAISE NOTICE '  - BYPASSRLS: 有効（全ギルドのデータにアクセス可能）';
    RAISE NOTICE '';
    RAISE NOTICE '次のステップ:';
    RAISE NOTICE '  1. .env.maintenanceファイルにパスワードを設定';
    RAISE NOTICE '  2. docker compose run --rm maintenance で実行';
    RAISE NOTICE '========================================';
END
$$;
