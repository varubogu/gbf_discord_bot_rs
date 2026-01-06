# ギルド設定（guild_settings）

## 概要

**テーブル物理名**: `guild_settings`
**スキーマ名**: `guild_master`
**テーブルタイプ**: Reference
**テーブルスコープ**: Guild（ギルド固有）
**実装状況**: ✅ 実装済み

## 用途

ギルド（Discordサーバー）ごとの設定を管理します。タイムゾーン、ロケール（言語設定）などのギルド単位の設定を保持します。

## カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|-----|------|------|
| guild_id | BIGINT | PK, NOT NULL | ギルドID（Discord Guild ID） |
| timezone | TEXT | NOT NULL | IANAタイムゾーン名（例: Asia/Tokyo, America/New_York） |
| locale | TEXT | NOT NULL | ロケール（言語設定、例: ja, en） |
| created_at | TIMESTAMPTZ | NOT NULL | 作成日時（UTC） |
| updated_at | TIMESTAMPTZ | NOT NULL | 更新日時（UTC） |

## 制約

### プライマリキー
- `guild_id`

### 外部キー
- `guild_id` → `guild_master.guilds(guild_id)`（論理的な参照）

### UNIQUE制約
なし

### NOT NULL制約
- `guild_id`, `timezone`, `locale`, `created_at`, `updated_at`

### CHECK制約
- `timezone` はIANAタイムゾーン名として有効な文字列
  - アプリケーション層で検証（chrono-tzでパース可能であること）

## インデックス

- **プライマリキーインデックス**: `guild_id`（自動作成）

## データサンプル

| guild_id | timezone | locale | created_at | updated_at |
|----------|----------|--------|------------|------------|
| 123456789 | Asia/Tokyo | ja | 2025-01-01 00:00:00+00 | 2025-01-01 00:00:00+00 |
| 987654321 | America/New_York | en | 2025-01-01 00:00:00+00 | 2025-01-15 12:30:00+00 |
| 555555555 | Europe/London | en | 2025-01-01 00:00:00+00 | 2025-01-01 00:00:00+00 |

## デフォルト動作

- ギルドの設定が存在しない場合、アプリケーション層で以下のデフォルト値を使用:
  - `timezone`: `Asia/Tokyo`（日本標準時）
  - `locale`: `ja`（日本語）
- DBには未登録状態でも動作可能

## タイムスタンプ自動更新

このテーブルは SeaORM の `ActiveModelBehavior` を使用して、以下のタイムスタンプが自動設定されます:

- **created_at**: レコード作成時に自動設定
- **updated_at**: レコード作成時・更新時に自動設定

詳細は [sea_orm_timestamp_automation.md](../../design/database/sea_orm_timestamp_automation.md) を参照してください。

## Rust実装

- **エンティティファイル**: `src/models/entities/guild_master/guild_settings.rs`
- **マイグレーションファイル**: `migration/src/m*_create_guild_settings.rs`
- **実装状況**: ✅ 実装済み

### エンティティ定義（抜粋）

```rust
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(schema_name = "guild_master", table_name = "guild_settings")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub guild_id: i64,
    pub timezone: String,
    pub locale: String,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}
```

## 備考

- **テーブル名変更**: 以前の設計では `guild_timezones` という名称でしたが、実装では `guild_settings` に統合されました
- **機能拡張**: タイムゾーン設定だけでなく、ロケール（言語設定）も追加されました
- **タイムゾーンの用途**:
  - ユーザー入力の日時解釈（例: 「21:00」→ギルドのタイムゾーンで解釈）
  - Discord上での日時表示（UTC → ギルドのタイムゾーンに変換して表示）
- **ロケールの用途**:
  - ボットの応答メッセージの言語選択
  - 多言語対応機能の基盤
- **関連コマンド**:
  - `/サーバー設定`: ギルドのタイムゾーンとロケールを設定

## 参照

サーバー設定機能の詳細は [timezone_settings.md](../../features/timezone_settings.md) を参照してください。
