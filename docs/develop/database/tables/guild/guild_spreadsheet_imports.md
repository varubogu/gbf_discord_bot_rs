# ギルドスプレッドシート取込設定（guild_spreadsheet_imports）

## 概要

**テーブル物理名**: `guild_spreadsheet_imports`  
**テーブルタイプ**: Reference  
**テーブルスコープ**: Guild

## 用途

ギルドごとの読み込み用（スプレッドシート→DB）スプレッドシートIDを管理します。`/gspread_load`コマンド実行時にこのテーブルからIDを取得し、指定されたスプレッドシートからギルド固有データをPostgreSQLへ取り込みます。

## カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|-----|------|------|
| guild_id | BigInteger | PK, NOT NULL | Discord Guild ID |
| spreadsheet_id | String(80) | NOT NULL | GoogleスプレッドシートID（`docs.google.com/spreadsheets/d/{id}`の`{id}`部分） |

> **備考**: 仕様要件に合わせ、補助カラム（memo, timestamp）は持たず、最小構成とする。

## 制約

### プライマリキー
- `guild_id`

### 外部キー
- なし（`guilds`テーブル参照はアプリケーション層で担保）

### UNIQUE制約
- なし（同一スプレッドシートを複数ギルドで共有可能）

## インデックス
- PKにより`guild_id`での検索を高速化

## データサンプル
| guild_id | spreadsheet_id |
|----------|----------------|
| 123456789012345678 | 1aBcDeFGhIJkLmNopQRstuVWxyz0123456789 |
| 987654321098765432 | 2bCdEfGHijKlmnOPqrSTUvwxYZ0987654321 |

## 関連テーブル
- `guild_spreadsheet_exports`: 書き込み用スプレッドシートIDを管理
- `guild_environments`: その他ギルド環境変数を管理（スプレッドシートIDは切り離し済み）

## 備考
- `/gspread_regist`コマンドでレコードをINSERT/UPDATE
- `/gspread_load`では必須依存。未登録の場合はPresentation層でエラー表示

## Rust実装
- **エンティティ**: `src/models/entities/guild_spreadsheet_imports.rs`（未実装）
- **Repository**: `GuildSpreadsheetImportRepository`（新規実装予定）
