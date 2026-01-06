# クエスト情報（quests）

## 概要

**テーブル物理名**: `quests`
**スキーマ名**: `master`
**テーブルタイプ**: Reference
**テーブルスコープ**: All（全ギルド共通）
**実装状況**: ✅ 実装済み

## 用途

グラブルのクエスト定義を管理します。クエスト名、募集人数、使用可能な戦術、デフォルト戦術などを定義し、マルチバトル募集の基準となる情報を提供します。

## カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|-----|------|------|
| id | INTEGER | PK, NOT NULL | クエストID（主キー） |
| name | TEXT | NOT NULL | クエスト名（例: プロトバハムート、アルバハHL） |
| default_battle_style_id | INTEGER | NOT NULL, FK | デフォルトの戦術ID（battle_styles.idを参照） |
| recruit_count | INTEGER | NOT NULL | 募集人数（マルチバトルの参加可能人数） |
| available_battle_style_ids | TEXT | NOT NULL | 使用可能な戦術ID（カンマ区切り、例: "1,2,3"） |
| sort_order | INTEGER | NOT NULL | ソート順序（表示順を制御） |
| created_at | TIMESTAMPTZ | NOT NULL | 作成日時（UTC） |
| updated_at | TIMESTAMPTZ | NOT NULL | 更新日時（UTC） |

## 制約

### プライマリキー
- `id`

### 外部キー
- `default_battle_style_id` → `master.battle_styles(id)`

### UNIQUE制約
なし

### NOT NULL制約
- `id`, `name`, `default_battle_style_id`, `recruit_count`, `available_battle_style_ids`, `sort_order`, `created_at`, `updated_at`

## インデックス

- **プライマリキーインデックス**: `id`（自動作成）
- **外部キーインデックス**: `default_battle_style_id`（外部キー制約で自動作成）

## データサンプル

| id | name | default_battle_style_id | recruit_count | available_battle_style_ids | sort_order |
|----|------|------------------------|--------------|---------------------------|-----------|
| 1 | プロトバハムート | 1 | 30 | 1,2,3 | 1 |
| 2 | アルティメットバハムートHL | 1 | 18 | 1,2 | 2 |
| 3 | ルシファーHL | 1 | 6 | 1 | 3 |

## 関連テーブル

### 参照元テーブル

- **master.quest_aliases**: `quest_id` で参照（1対多）
- **worker.battle_recruitments**: `quest_id` で参照（1対多）

### 参照先テーブル

- **master.battle_styles**: `default_battle_style_id` で参照（多対1）

## タイムスタンプ自動更新

このテーブルは SeaORM の `ActiveModelBehavior` を使用して、以下のタイムスタンプが自動設定されます:

- **created_at**: レコード作成時に自動設定
- **updated_at**: レコード作成時・更新時に自動設定

詳細は [sea_orm_timestamp_automation.md](../../design/database/sea_orm_timestamp_automation.md) を参照してください。

## Rust実装

- **エンティティファイル**: `src/models/entities/master/quests.rs`
- **マイグレーションファイル**: `migration/src/m*_create_quests.rs`
- **実装状況**: ✅ 実装済み

### エンティティ定義（抜粋）

```rust
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(schema_name = "master", table_name = "quests")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub default_battle_style_id: i32,
    pub recruit_count: i32,
    pub available_battle_style_ids: String,
    pub sort_order: i32,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}
```

## 備考

- **カラム名変更**: `target_id` → `id`, `quest_name` → `name`, `use_battle_type` → `available_battle_style_ids`, `default_battle_type` → `default_battle_style_id` に変更
- **追加カラム**: `sort_order` が追加され、表示順序を制御できるようになりました
- **外部キー追加**: `default_battle_style_id` に外部キー制約が追加されました
- マルチバトル募集の基準となるクエスト情報を定義
- `available_battle_style_ids` は文字列としてbattle_stylesのIDをカンマ区切りで保持
- `recruit_count` はマルチバトルの最大参加人数を示す
- `sort_order` により、表示順序をカスタマイズ可能
