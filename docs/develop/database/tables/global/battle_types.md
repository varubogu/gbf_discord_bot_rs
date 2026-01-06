# マルチバトル戦術（battle_styles）

## 概要

**テーブル物理名**: `battle_styles`
**スキーマ名**: `master`
**テーブルタイプ**: Reference
**テーブルスコープ**: All（全ギルド共通）
**実装状況**: ✅ 実装済み

## 用途

グラブルのマルチバトルにおける戦術タイプ（青箱優先、トレハン優先など）を定義します。Discordリアクション（絵文字）とバトルスタイルを紐づけ、ユーザーがリアクションで戦術を選択できるようにします。

## カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|-----|------|------|
| id | INTEGER | PK, NOT NULL | 戦術ID（主キー） |
| display_name | TEXT | NOT NULL | 戦術の表示名（例: 青箱優先、トレハン優先） |
| reactions | TEXT | NULLABLE | 戦術に応じたDiscordリアクション（絵文字） |
| sort_order | INTEGER | NOT NULL | ソート順序（表示順を制御） |
| created_at | TIMESTAMPTZ | NOT NULL | 作成日時（UTC） |
| updated_at | TIMESTAMPTZ | NOT NULL | 更新日時（UTC） |

## 制約

### プライマリキー
- `id`

### 外部キー
なし

### UNIQUE制約
なし

### NOT NULL制約
- `id`, `display_name`, `sort_order`, `created_at`, `updated_at`

## インデックス

- **プライマリキーインデックス**: `id`（自動作成）

## データサンプル

| id | display_name | reactions | sort_order |
|----|-------------|-----------|-----------|
| 1 | 青箱優先 | 🔵 | 1 |
| 2 | トレハン優先 | 💎 | 2 |
| 3 | 速攻 | ⚡ | 3 |

## 関連テーブル

### 参照元テーブル

- **master.quests**: `default_battle_style_id` で参照（1対多）
- **worker.battle_recruitments**: `battle_style_id` で参照（1対多）

## タイムスタンプ自動更新

このテーブルは SeaORM の `ActiveModelBehavior` を使用して、以下のタイムスタンプが自動設定されます:

- **created_at**: レコード作成時に自動設定
- **updated_at**: レコード作成時・更新時に自動設定

詳細は [sea_orm_timestamp_automation.md](../../design/database/sea_orm_timestamp_automation.md) を参照してください。

## Rust実装

- **エンティティファイル**: `src/models/entities/master/battle_styles.rs`
- **マイグレーションファイル**: `migration/src/m*_create_battle_styles.rs`
- **実装状況**: ✅ 実装済み

### エンティティ定義（抜粋）

```rust
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "battle_styles", schema_name = "master")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub display_name: String,
    pub reactions: Option<String>,
    pub sort_order: i32,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}
```

## 備考

- **テーブル名変更**: 以前の設計では `battle_types` という名称でしたが、実装では `battle_styles` に統一されています
- **カラム名変更**: `type_id` → `id`, `name` → `display_name` に変更
- **追加カラム**: `sort_order` が追加され、表示順序を制御できるようになりました
- マルチバトル募集時に、ユーザーが選択可能な戦術を定義
- Discordリアクションとの紐づけにより、直感的な戦術選択が可能
- `sort_order` により、表示順序をカスタマイズ可能

## スプレッドシート連携

このテーブルは、グローバルスプレッドシートから読み込み可能です。

- **コマンド**: `/gspread_global_load`（Bot管理者専用）
- **シート名**: 「マルチバトル戦術」または「battle_styles」
- **対象カラム**: `id`, `display_name`, `reactions`, `sort_order`

詳細は [google_spreadsheet.md](../../features/google_spreadsheet.md) を参照してください。
