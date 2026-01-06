# 環境変数（environments）

## 概要

**テーブル物理名**: `environments`
**スキーマ名**: `master`
**テーブルタイプ**: Reference
**テーブルスコープ**: All（全ギルド共通）
**実装状況**: ✅ 実装済み

## 用途

Bot動作設定のグローバル環境変数を管理します。アプリケーション全体の設定値をデータベースで管理し、再起動なしに設定変更が可能です。

## カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|-----|------|------|
| key | TEXT | PK, NOT NULL | 環境変数のキー（例: LOG_LEVEL、MAX_RETRIES） |
| value | TEXT | NOT NULL | 環境変数の値 |
| created_at | TIMESTAMPTZ | NOT NULL | 作成日時（UTC） |
| updated_at | TIMESTAMPTZ | NOT NULL | 更新日時（UTC） |

## 制約

### プライマリキー
- `key`

### 外部キー
なし

### UNIQUE制約
なし

### NOT NULL制約
- `key`, `value`, `created_at`, `updated_at`

## インデックス

- **プライマリキーインデックス**: `key`（自動作成）

## データサンプル

| key | value |
|-----|-------|
| LOG_LEVEL | INFO |
| MAX_RETRIES | 3 |
| RECRUITMENT_EXPIRY_HOURS | 24 |

## 関連テーブル

### 関連テーブル

- **worker.guild_environments**: ギルド固有の環境変数（オーバーライド用）

## タイムスタンプ自動更新

このテーブルは SeaORM の `ActiveModelBehavior` を使用して、以下のタイムスタンプが自動設定されます:

- **created_at**: レコード作成時に自動設定
- **updated_at**: レコード作成時・更新時に自動設定

詳細は [sea_orm_timestamp_automation.md](../../design/database/sea_orm_timestamp_automation.md) を参照してください。

## Rust実装

- **エンティティファイル**: `src/models/entities/master/environments.rs`
- **マイグレーションファイル**: `migration/src/m*_create_environments.rs`
- **実装状況**: ✅ 実装済み

### エンティティ定義（抜粋）

```rust
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(schema_name = "master", table_name = "environments")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub key: String,
    pub value: String,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}
```

## 備考

- **カラム削除**: `memo` カラムは実装に存在しないため削除されました
- **NOT NULL変更**: `value` が NULLABLE から NOT NULL に変更されました
- グローバル設定として全ギルドに適用
- `guild_environments` で上書き可能
- 設定変更時はアプリケーション再起動不要
