#!/bin/bash
set -e

# ================================
# GBF Discord Bot データベース削除スクリプト
# ================================
# データベースとロールを削除します
# ================================

echo "=========================================="
echo "データベース削除開始"
echo "=========================================="

# 環境変数の確認（デフォルト値を設定）
DB_HOST="${DB_HOST:-localhost}"
DB_USER="${DB_USER:-postgres}"

echo "接続先: ${DB_HOST}"

# drop.sql を実行
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

psql -v ON_ERROR_STOP=1 \
     -h "$DB_HOST" \
     -U "$DB_USER" \
     -d postgres \
     -f "$SCRIPT_DIR/drop.sql"

echo "=========================================="
echo "データベース削除が完了しました"
echo "=========================================="
