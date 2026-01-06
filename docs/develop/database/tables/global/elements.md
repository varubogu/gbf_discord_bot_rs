# 属性定義（elements）

## 概要

**テーブル物理名**: `elements`
**スキーマ名**: `master`
**テーブルタイプ**: Reference
**テーブルスコープ**: All（全ギルド共通）
**実装状況**: ✅ 実装済み

## 用途

グラブルのゲーム内属性（火、水、土、風、光、闇）を定義します。イベントスケジュールの有利属性表示や、ゲーム要素の分類に使用されます。

## カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|-----|------|------|
| id | INTEGER | PK, NOT NULL | 属性ID（主キー） |
| reaction_stamp | TEXT | NULLABLE | 属性リアクションスタンプ（絵文字、例: 🔥） |
| name_jp | TEXT | NOT NULL | 属性名（日本語、例: 火） |
| name_en | TEXT | NULLABLE | 属性名（英語、例: Fire） |

## 制約

### プライマリキー
- `id`

### 外部キー
なし

### UNIQUE制約
なし

### NOT NULL制約
- `id`, `name_jp`

## インデックス

- **プライマリキーインデックス**: `id`（自動作成）

## データサンプル

| id | reaction_stamp | name_jp | name_en |
|----|---------------|---------|---------|
| 1 | 🔥 | 火 | Fire |
| 2 | 💧 | 水 | Water |
| 3 | 🌍 | 土 | Earth |
| 4 | 💨 | 風 | Wind |
| 5 | ⭐ | 光 | Light |
| 6 | 🌙 | 闇 | Dark |

## 関連テーブル

### 参照元テーブル

- **master.event_schedules**: `weak_attribute` で参照（1対多）

## Rust実装

- **エンティティファイル**: `src/models/entities/master/elements.rs`
- **マイグレーションファイル**: `migration/src/m*_create_elements.rs`
- **実装状況**: ✅ 実装済み

### エンティティ定義（抜粋）

```rust
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(schema_name = "master", table_name = "elements")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub reaction_stamp: Option<String>,
    pub name_jp: String,
    pub name_en: Option<String>,
}
```

## 備考

- **カラム名変更**: `element_id` → `id`, `stamp` → `reaction_stamp` に変更
- **NOT NULL変更**: `name_jp` が NULLABLE から NOT NULL に変更されました
- グラブルの6属性を定義
- `reaction_stamp` は Discord上での表示用絵文字
- 多言語対応のため日本語と英語の両方を保持
- タイムスタンプカラムは含まれません（シンプルなマスタテーブル）
