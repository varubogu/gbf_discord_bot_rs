# チャンネル種類（channel_types）

## 概要

**テーブル物理名**: `channel_types`
**スキーマ名**: `master`
**テーブルタイプ**: Reference
**テーブルスコープ**: All（全ギルド共通）
**実装状況**: ✅ 実装済み

## 用途

Discordチャンネルの用途分類を定義します。募集チャンネル、通知チャンネルなど、チャンネルの役割を管理します。

## カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|-----|------|------|
| id | INTEGER | PK, NOT NULL | チャンネル種類ID（主キー、auto_increment = false） |
| name | TEXT | NOT NULL | チャンネル種類名（例: 募集チャンネル、通知チャンネル） |
| memo | TEXT | NULLABLE | メモ |

## 制約

### プライマリキー
- `id`（auto_increment = false）

### 外部キー
なし

### UNIQUE制約
なし

### NOT NULL制約
- `id`, `name`

## インデックス

- **プライマリキーインデックス**: `id`（自動作成）

## データサンプル

| id | name | memo |
|----|------|------|
| 1 | 募集チャンネル | マルチバトル募集用 |
| 2 | 通知チャンネル | イベント通知用 |
| 3 | 管理チャンネル | Bot管理用 |

## 関連テーブル

### 参照元テーブル

- **worker.guild_channels**: `channel_type` で参照（1対多）

## Rust実装

- **エンティティファイル**: `src/models/entities/master/channel_types.rs`
- **マイグレーションファイル**: `migration/src/m*_create_channel_types.rs`
- **実装状況**: ✅ 実装済み

### エンティティ定義（抜粋）

```rust
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(schema_name = "master", table_name = "channel_types")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i32,
    pub name: String,
    pub memo: Option<String>,
}
```

## 備考

- **カラム名変更**: `channel_type` → `id`, `channel_type_name` → `name` に変更
- 主キーは `auto_increment = false` として定義されています（手動でID管理）
- チャンネルの役割を定義し、Botの動作を制御
- `guild_channels` テーブルで具体的なチャンネルと紐づけ
- タイムスタンプカラムは含まれません（シンプルなマスタテーブル）
