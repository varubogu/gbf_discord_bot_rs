#!/bin/bash
set -e

# ================================
# GBF Discord Bot データベース初期化スクリプト
# ================================
# 環境変数からパスワードを読み込んで init.sql に渡します
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
DB_HOST="${DB_HOST:-localhost}"
DB_USER="${DB_USER:-postgres}"

echo "データベース名: ${DB_NAME}"
echo "接続先: ${DB_HOST}"
echo "環境変数から以下のロールのパスワードを読み込みました:"
echo "  - SYSTEM_DB_PASSWORD: $([ -n "$SYSTEM_DB_PASSWORD" ] && echo '設定済み' || echo '未設定')"
echo "  - GUILD_DB_PASSWORD: $([ -n "$GUILD_DB_PASSWORD" ] && echo '設定済み' || echo '未設定')"
echo "  - GLOBAL_DB_PASSWORD: $([ -n "$GLOBAL_DB_PASSWORD" ] && echo '設定済み' || echo '未設定')"
echo "  - ADMIN_DB_PASSWORD: $([ -n "$ADMIN_DB_PASSWORD" ] && echo '設定済み' || echo '未設定')"

# init.sql を実行（psql変数で環境変数を渡す）
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# 最速だとpostgresの準備ができていないため待機
sleep 30

psql -v ON_ERROR_STOP=1 \
     -U "$DB_USER" \
     -d postgres \
     -v system_password="$SYSTEM_DB_PASSWORD" \
     -v guild_password="$GUILD_DB_PASSWORD" \
     -v global_password="$GLOBAL_DB_PASSWORD" \
     -v admin_password="$ADMIN_DB_PASSWORD" \
     -v db_name="$DB_NAME" \
     -f "$SCRIPT_DIR/../sql/init.sql"

echo "=========================================="
echo "データベース初期化が正常に完了しました"
echo "=========================================="
