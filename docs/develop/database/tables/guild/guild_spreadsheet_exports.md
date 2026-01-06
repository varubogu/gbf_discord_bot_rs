# ギルドスプレッドシート出力設定（guild_spreadsheet_exports）

## 概要

**テーブル物理名**: `guild_spreadsheet_exports`  
**テーブルタイプ**: Reference  
**テーブルスコープ**: Guild

## 用途

ギルドごとの書き込み用（DB→スプレッドシート）スプレッドシートIDを管理します。`/gspread_push`コマンドでPostgreSQL→スプレッドシート同期を行う際、このテーブルから書き込み先IDを取得します。

## カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|-----|------|------|
| guild_id | BigInteger | PK, NOT NULL | Discord Guild ID |
| spreadsheet_id | String(80) | NOT NULL | GoogleスプレッドシートID（`docs.google.com/spreadsheets/d/{id}`の`{id}`部分） |

> **備考**: 読み込みテーブルと同様、テーブル要件に従い最小構成で保持します。

## 制約

### プライマリキー
- `guild_id`

### 外部キー
- なし

### UNIQUE制約
- なし（複数ギルドで同じスプレッドシートに書き込みたい場合も想定）

## インデックス
- PKにより`guild_id`検索を最適化

## データサンプル
| guild_id | spreadsheet_id |
|----------|----------------|
| 123456789012345678 | 1aBcDeFGhIJkLmNopQRstuVWxyz0123456789 |
| 987654321098765432 | 5FgHiJKlmNoPQRstuVWXyZ0123456789abcde |

## 関連テーブル
- `guild_spreadsheet_imports`: 読み込み先スプレッドシートID
- `guild_environments`: その他ギルド設定テーブル

## 備考
- `/gspread_regist`コマンドでINSERT/UPDATE
- `/gspread_push`実行前に必須チェックを行い、未登録ならエラー応答

## Rust実装
- **エンティティ**: `src/models/entities/guild_spreadsheet_exports.rs`（未実装）
- **Repository**: `GuildSpreadsheetExportRepository`（新規実装予定）
