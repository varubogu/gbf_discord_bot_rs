#!/bin/bash
set -e

# ================================
# GBF Discord Bot データベース初期化スクリプト
# ================================
# 環境変数から各ロールのパスワードを読み込んで
# データベースとロールを作成します
# ================================

echo "=========================================="
echo "GBF Discord Bot データベース初期化開始"
echo "=========================================="

# 環境変数の確認（デフォルト値を設定）
SYSTEM_DB_PASSWORD="${SYSTEM_DB_PASSWORD:-change_this_system_password}"
GUILD_DB_PASSWORD="${GUILD_DB_PASSWORD:-change_this_guild_password}"
GLOBAL_DB_PASSWORD="${GLOBAL_DB_PASSWORD:-change_this_global_password}"
ADMIN_DB_PASSWORD="${ADMIN_DB_PASSWORD:-change_this_admin_password}"
DB_NAME="${DB_NAME:-gbf_bot_db}"

echo "データベース名: ${DB_NAME}"
echo "環境変数から以下のロールのパスワードを読み込みました:"
echo "  - SYSTEM_DB_PASSWORD: $([ -n "$SYSTEM_DB_PASSWORD" ] && echo '設定済み' || echo '未設定')"
echo "  - GUILD_DB_PASSWORD: $([ -n "$GUILD_DB_PASSWORD" ] && echo '設定済み' || echo '未設定')"
echo "  - GLOBAL_DB_PASSWORD: $([ -n "$GLOBAL_DB_PASSWORD" ] && echo '設定済み' || echo '未設定')"
echo "  - ADMIN_DB_PASSWORD: $([ -n "$ADMIN_DB_PASSWORD" ] && echo '設定済み' || echo '未設定')"

# PostgreSQLにSQLを実行
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    -- ================================
    -- ロール作成
    -- ================================

    -- 1. System ロール（スケジューラー、バックグラウンドタスク用）
    DO \$\$
    BEGIN
        IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'gbf_bot_system') THEN
            CREATE ROLE gbf_bot_system WITH LOGIN PASSWORD '${SYSTEM_DB_PASSWORD}';
            RAISE NOTICE 'ロール gbf_bot_system を作成しました';
        ELSE
            ALTER ROLE gbf_bot_system WITH PASSWORD '${SYSTEM_DB_PASSWORD}';
            RAISE NOTICE 'ロール gbf_bot_system のパスワードを更新しました';
        END IF;
    END
    \$\$;

    -- System ロールはRLSをバイパス（全ギルドのデータにアクセス可能）
    ALTER ROLE gbf_bot_system WITH BYPASSRLS;

    -- 2. Guild ロール（通常のDiscordコマンド実行用）
    DO \$\$
    BEGIN
        IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'gbf_bot_guild') THEN
            CREATE ROLE gbf_bot_guild WITH LOGIN PASSWORD '${GUILD_DB_PASSWORD}';
            RAISE NOTICE 'ロール gbf_bot_guild を作成しました';
        ELSE
            ALTER ROLE gbf_bot_guild WITH PASSWORD '${GUILD_DB_PASSWORD}';
            RAISE NOTICE 'ロール gbf_bot_guild のパスワードを更新しました';
        END IF;
    END
    \$\$;

    -- Guild ロールはRLSポリシーに従う（BYPASSRLSなし）
    -- app.current_guild_id でギルドごとにデータを分離

    -- 3. Global ロール（マスターデータ更新、スプレッドシート同期用）
    DO \$\$
    BEGIN
        IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'gbf_bot_global') THEN
            CREATE ROLE gbf_bot_global WITH LOGIN PASSWORD '${GLOBAL_DB_PASSWORD}';
            RAISE NOTICE 'ロール gbf_bot_global を作成しました';
        ELSE
            ALTER ROLE gbf_bot_global WITH PASSWORD '${GLOBAL_DB_PASSWORD}';
            RAISE NOTICE 'ロール gbf_bot_global のパスワードを更新しました';
        END IF;
    END
    \$\$;

    -- Global ロールはRLSをバイパス（マスターデータ更新のため）
    ALTER ROLE gbf_bot_global WITH BYPASSRLS;

    -- 4. Admin ロール（マイグレーション実行、管理操作用）
    DO \$\$
    BEGIN
        IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'gbf_bot_admin') THEN
            CREATE ROLE gbf_bot_admin WITH LOGIN PASSWORD '${ADMIN_DB_PASSWORD}';
            RAISE NOTICE 'ロール gbf_bot_admin を作成しました';
        ELSE
            ALTER ROLE gbf_bot_admin WITH PASSWORD '${ADMIN_DB_PASSWORD}';
            RAISE NOTICE 'ロール gbf_bot_admin のパスワードを更新しました';
        END IF;
    END
    \$\$;

    -- Admin ロールはRLSをバイパス＋スキーマ作成権限
    ALTER ROLE gbf_bot_admin WITH BYPASSRLS CREATEDB;
EOSQL

# データベース作成（既存の場合はスキップ）
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    SELECT 'CREATE DATABASE ${DB_NAME} WITH OWNER gbf_bot_admin ENCODING ''UTF8'''
    WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = '${DB_NAME}')\gexec
EOSQL

# 作成したデータベースに接続して権限付与
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "${DB_NAME}" <<-EOSQL
    -- ================================
    -- 権限付与
    -- ================================

    -- データベースへの接続権限
    GRANT CONNECT ON DATABASE ${DB_NAME} TO gbf_bot_system;
    GRANT CONNECT ON DATABASE ${DB_NAME} TO gbf_bot_guild;
    GRANT CONNECT ON DATABASE ${DB_NAME} TO gbf_bot_global;
    GRANT CONNECT ON DATABASE ${DB_NAME} TO gbf_bot_admin;

    -- スキーマ作成権限（adminロールのみ）
    -- マイグレーション実行時に必要
    GRANT CREATE ON DATABASE ${DB_NAME} TO gbf_bot_admin;

    -- ================================
    -- 完了メッセージ
    -- ================================
    DO \$\$
    BEGIN
        RAISE NOTICE '========================================';
        RAISE NOTICE 'GBF Discord Bot データベース初期化完了';
        RAISE NOTICE '========================================';
        RAISE NOTICE 'データベース: ${DB_NAME}';
        RAISE NOTICE 'ロール:';
        RAISE NOTICE '  - gbf_bot_system (BYPASSRLS)';
        RAISE NOTICE '  - gbf_bot_guild (RLS適用)';
        RAISE NOTICE '  - gbf_bot_global (BYPASSRLS)';
        RAISE NOTICE '  - gbf_bot_admin (BYPASSRLS, CREATEDB)';
        RAISE NOTICE '';
        RAISE NOTICE '次のステップ:';
        RAISE NOTICE '  1. アプリケーションコンテナでマイグレーション実行';
        RAISE NOTICE '========================================';
    END
    \$\$;
EOSQL

echo "=========================================="
echo "データベース初期化が正常に完了しました"
echo "=========================================="
