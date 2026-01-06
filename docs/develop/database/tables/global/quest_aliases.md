# クエスト別名（quest_aliases）

## 概要

**テーブル物理名**: `quest_aliases`
**スキーマ名**: `master`
**テーブルタイプ**: Reference
**テーブルスコープ**: All（全ギルド共通）
**実装状況**: ✅ 実装済み

## 用途

クエストの別名・略称を定義し、ユーザーが様々な表記でクエストを指定できるようにします。検索機能や自動認識機能で使用されます。

## カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|-----|------|------|
| quest_id | INTEGER | PK, NOT NULL, FK | クエストID（quests.idを参照） |
| sequence_no | INTEGER | PK, NOT NULL | 別名連番ID |
| alias | TEXT | UNIQUE, NOT NULL | クエスト別名（例: アルバハHL、プロバハ） |
| alias_kana_small | TEXT | NOT NULL | クエスト別名（半角カナ）検索用 |
| created_at | TIMESTAMPTZ | NOT NULL | 作成日時（UTC） |
| updated_at | TIMESTAMPTZ | NOT NULL | 更新日時（UTC） |

## 制約

### プライマリキー
- `quest_id`, `sequence_no`（複合キー、auto_increment = false）

### 外部キー
- `quest_id` → `master.quests(id)`

### UNIQUE制約
- UNIQUE(`alias`) - 制約名: unique_quest_alias

### NOT NULL制約
- `quest_id`, `sequence_no`, `alias`, `alias_kana_small`, `created_at`, `updated_at`

## インデックス

- **プライマリキーインデックス**: `quest_id`, `sequence_no`（自動作成）
- **ユニークインデックス**: `alias`（自動作成）
- **外部キーインデックス**: `quest_id`（外部キー制約で自動作成）

## データサンプル

| quest_id | sequence_no | alias | alias_kana_small |
|----------|------------|-------|------------------|
| 1 | 1 | プロトバハムート | ﾌﾟﾛﾄﾊﾞﾊﾑｰﾄ |
| 1 | 2 | プロバハ | ﾌﾟﾛﾊﾞﾊ |
| 2 | 1 | アルティメットバハムートHL | ｱﾙﾃｨﾒｯﾄﾊﾞﾊﾑｰﾄHL |
| 2 | 2 | アルバハHL | ｱﾙﾊﾞﾊHL |

## 関連テーブル

### 参照先テーブル

- **master.quests**: `quest_id` で参照（多対1）

## タイムスタンプ自動更新

このテーブルは SeaORM の `ActiveModelBehavior` を使用して、以下のタイムスタンプが自動設定されます:

- **created_at**: レコード作成時に自動設定
- **updated_at**: レコード作成時・更新時に自動設定

詳細は [sea_orm_timestamp_automation.md](../../design/database/sea_orm_timestamp_automation.md) を参照してください。

## Rust実装

- **エンティティファイル**: `src/models/entities/master/quest_aliases.rs`
- **マイグレーションファイル**: `migration/src/m*_create_quest_aliases.rs`
- **実装状況**: ✅ 実装済み

### エンティティ定義（抜粋）

```rust
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(schema_name = "master", table_name = "quest_aliases")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub quest_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub sequence_no: i32,
    pub alias: String,
    pub alias_kana_small: String,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}
```

## 備考

- **カラム名変更**: `target_id` → `quest_id`, `target_alias_id` → `sequence_no` に変更
- **NOT NULL変更**: `alias_kana_small` が NULLABLE から NOT NULL に変更されました
- 複合主キーは `auto_increment = false` として定義されています
- 1つのクエストに対して複数の別名を登録可能
- `alias_kana_small` は検索時のカナ表記マッチング用
- 別名はシステム全体で一意でなければならない
