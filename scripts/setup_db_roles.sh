#!/bin/bash
set -euo pipefail

# 使用方法表示
usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --env-file PATH    .envファイルのパスを指定（省略時は環境変数を使用）"
    echo "  -h, --help         このヘルプメッセージを表示"
    echo ""
    echo "Examples:"
    echo "  $0 --env-file .env"
    echo "  $0 --env-file /path/to/.env.production"
    echo "  source .env && $0"
    exit 0
}

# コマンドライン引数解析
ENV_FILE=""
while [[ $# -gt 0 ]]; do
    case $1 in
        --env-file)
            ENV_FILE="$2"
            shift 2
            ;;
        -h|--help)
            usage
            ;;
        *)
            echo "Error: Unknown option: $1"
            usage
            ;;
    esac
done

# .envファイルが指定されている場合は読み込み
if [ -n "$ENV_FILE" ]; then
    if [ ! -f "$ENV_FILE" ]; then
        echo "Error: .env file not found: $ENV_FILE"
        exit 1
    fi
    echo "Loading environment variables from: $ENV_FILE"
    # コメント行と空行を除外して読み込み
    set -a
    source <(grep -v '^#' "$ENV_FILE" | grep -v '^$' | sed 's/^[[:space:]]*//' | grep -v '^#')
    set +a
fi

# 環境変数から接続情報取得
DB_HOST=${DB_HOST:-localhost}
DB_NAME=${DB_NAME:-gbf_bot}
DB_ADMIN_USER=${DB_ADMIN_USER:-postgres}

# パスワード環境変数の存在確認
if [ -z "${SYSTEM_DB_PASSWORD:-}" ]; then
    echo "Error: SYSTEM_DB_PASSWORD is not set"
    exit 1
fi

if [ -z "${GUILD_DB_PASSWORD:-}" ]; then
    echo "Error: GUILD_DB_PASSWORD is not set"
    exit 1
fi

if [ -z "${GLOBAL_DB_PASSWORD:-}" ]; then
    echo "Error: GLOBAL_DB_PASSWORD is not set"
    exit 1
fi

if [ -z "${ADMIN_DB_PASSWORD:-}" ]; then
    echo "Error: ADMIN_DB_PASSWORD is not set"
    exit 1
fi

echo "Creating database roles..."
echo "DB_HOST: $DB_HOST"
echo "DB_NAME: $DB_NAME"
echo "DB_ADMIN_USER: $DB_ADMIN_USER"

psql -h "$DB_HOST" -U "$DB_ADMIN_USER" -d "$DB_NAME" <<EOF
-- ロール作成（既に存在する場合はスキップ）
DO \$\$
BEGIN
    -- gbf_bot_system ロール（スケジューラー・システム処理用）
    IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'gbf_bot_system') THEN
        CREATE ROLE gbf_bot_system BYPASSRLS LOGIN PASSWORD '${SYSTEM_DB_PASSWORD}';
        RAISE NOTICE 'Created role: gbf_bot_system';
    ELSE
        RAISE NOTICE 'Role already exists: gbf_bot_system';
    END IF;

    -- gbf_bot_guild ロール（通常のコマンド実行用）
    IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'gbf_bot_guild') THEN
        CREATE ROLE gbf_bot_guild LOGIN PASSWORD '${GUILD_DB_PASSWORD}';
        RAISE NOTICE 'Created role: gbf_bot_guild';
    ELSE
        RAISE NOTICE 'Role already exists: gbf_bot_guild';
    END IF;

    -- gbf_bot_global ロール（統計・スプレッドシート更新用）
    IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'gbf_bot_global') THEN
        CREATE ROLE gbf_bot_global BYPASSRLS LOGIN PASSWORD '${GLOBAL_DB_PASSWORD}';
        RAISE NOTICE 'Created role: gbf_bot_global';
    ELSE
        RAISE NOTICE 'Role already exists: gbf_bot_global';
    END IF;

    -- gbf_bot_admin ロール（管理・マイグレーション用）
    IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'gbf_bot_admin') THEN
        CREATE ROLE gbf_bot_admin BYPASSRLS LOGIN PASSWORD '${ADMIN_DB_PASSWORD}';
        RAISE NOTICE 'Created role: gbf_bot_admin';
    ELSE
        RAISE NOTICE 'Role already exists: gbf_bot_admin';
    END IF;
END
\$\$;

EOF

# データベース接続権限（別のSQL文で実行）
psql -h "$DB_HOST" -U "$DB_ADMIN_USER" -d "$DB_NAME" -c \
    "GRANT CONNECT ON DATABASE $DB_NAME TO gbf_bot_system, gbf_bot_guild, gbf_bot_global, gbf_bot_admin;" || {
    echo "Warning: Failed to grant CONNECT privilege. Roles may already have it."
}

# データベースレベルのCREATE権限付与（スキーマ作成用）
echo "Granting CREATE permission on database..."
psql -h "$DB_HOST" -U "$DB_ADMIN_USER" -d "$DB_NAME" -c \
    "GRANT CREATE ON DATABASE $DB_NAME TO gbf_bot_admin;" || {
    echo "Warning: Failed to grant CREATE privilege on database."
}

# publicスキーマへの権限付与（マイグレーション用）
echo "Granting permissions on public schema..."
psql -h "$DB_HOST" -U "$DB_ADMIN_USER" -d "$DB_NAME" <<EOF
-- publicスキーマへの権限付与
GRANT USAGE, CREATE ON SCHEMA public TO gbf_bot_admin;
GRANT ALL ON ALL TABLES IN SCHEMA public TO gbf_bot_admin;
GRANT ALL ON ALL SEQUENCES IN SCHEMA public TO gbf_bot_admin;

-- 将来作成されるテーブル・シーケンスへのデフォルト権限
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON TABLES TO gbf_bot_admin;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON SEQUENCES TO gbf_bot_admin;
EOF

echo "Database roles created successfully!"
echo ""
echo "Created roles:"
echo "  - gbf_bot_system: BYPASSRLS, master SELECT only"
echo "  - gbf_bot_guild:  RLS applied, guild-scoped access"
echo "  - gbf_bot_global: BYPASSRLS, full CRUD on all schemas"
echo "  - gbf_bot_admin:  BYPASSRLS, full admin privileges"
echo ""
echo "Next steps:"
echo "1. Update .env file with all role credentials (see .env.example)"
echo "2. Run migrations:"
if [ -n "$ENV_FILE" ]; then
    echo "   source $ENV_FILE"
fi
echo "   cargo run --bin migration"
echo ""
echo "3. Run application (uses DbRole::Guild by default):"
echo "   cargo run"
