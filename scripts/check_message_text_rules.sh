#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "[check] MessageTextId 文字列キー直渡しを検査します"
if rg -n 'get_message_from_context\([^)]*"' src; then
  echo "[error] get_message_from_context に文字列キーを直接渡している箇所があります。"
  exit 1
fi

echo "[check] ctx.say への文字列リテラル直書きを検査します"
if rg -n 'ctx\.say\("' src/events; then
  echo "[error] ctx.say に文字列リテラルを直接渡している箇所があります。"
  exit 1
fi

echo "[ok] message_text ルール検査に合格しました"
