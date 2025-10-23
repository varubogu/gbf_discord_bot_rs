# 環境変数（environments）

## 概要

**テーブル物理名**: `environments`
**テーブルタイプ**: Reference
**テーブルスコープ**: All

## 用途

Bot動作設定のグローバル環境変数を管理します。アプリケーション全体の設定値をデータベースで管理し、再起動なしに設定変更が可能です。

## カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|-----|------|------|
| key | String | PK, NOT NULL | 環境変数のキー（例: LOG_LEVEL、MAX_RETRIES） |
| value | String | NULLABLE | 環境変数の値 |
| memo | String | NULLABLE | メモ（設定の説明） |

## 制約

### プライマリキー
- `key`

### 外部キー
なし

### UNIQUE制約
なし

## インデックス

- PK: `key`（自動作成）

## データサンプル

| key | value | memo |
|-----|-------|------|
| LOG_LEVEL | INFO | ログレベル設定 |
| MAX_RETRIES | 3 | API呼び出し最大リトライ回数 |
| RECRUITMENT_EXPIRY_HOURS | 24 | 募集の有効期限（時間） |

## 関連テーブル

- **関連**: `guild_environments`（ギルド固有の環境変数）

## 備考

- グローバル設定として全ギルドに適用
- guild_environmentsで上書き可能
- 設定変更時はアプリケーション再起動不要

## Rust実装

- **エンティティ**: `src/models/entities/environments.rs`
- **実装状況**: 実装済み
