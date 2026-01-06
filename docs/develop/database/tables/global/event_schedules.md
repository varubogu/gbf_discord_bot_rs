# イベントスケジュール（event_schedules）

## 概要

**テーブル物理名**: `event_schedules`
**スキーマ名**: `master`
**テーブルタイプ**: Reference
**テーブルスコープ**: All（全ギルド共通）
**実装状況**: ✅ 実装済み

## 用途

グラブルイベントの開催スケジュールを定義します。イベント種類、開催回数、期間、有利属性などを管理し、詳細スケジュールとの紐づけを行います。

## カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|-----|------|------|
| id | UUID | PK, NOT NULL | 行ID（プライマリキー、auto_increment = false） |
| event_type | TEXT | NOT NULL, UNIQUE(event_type, event_count) | イベント種類（例: 古戦場、ゼノ撃滅戦） |
| event_count | BIGINT | NOT NULL, UNIQUE(event_type, event_count) | イベント開催回数（第N回） |
| profile | TEXT | NOT NULL | イベント詳細スケジュールとの紐づけプロファイル |
| weak_attribute | INTEGER | NOT NULL | 有利属性ID（外部キー制約なし） |
| start_at | TIMESTAMP | NOT NULL | 開始日時（タイムゾーンなし、JST想定） |
| end_at | TIMESTAMP | NOT NULL | 終了日時（タイムゾーンなし、JST想定） |
| created_at | TIMESTAMPTZ | NOT NULL | 作成日時（UTC） |
| updated_at | TIMESTAMPTZ | NOT NULL | 更新日時（UTC） |

## 制約

### プライマリキー
- `id`（auto_increment = false）

### 外部キー
なし（`weak_attribute` は外部キー制約なし）

### UNIQUE制約
- UNIQUE(`event_type`, `event_count`) - 制約名: unique_event_schedule

### NOT NULL制約
- `id`, `event_type`, `event_count`, `profile`, `weak_attribute`, `start_at`, `end_at`, `created_at`, `updated_at`

## インデックス

- **プライマリキーインデックス**: `id`（自動作成）
- **ユニークインデックス**: `event_type`, `event_count`（自動作成）

## データサンプル

| id | event_type | event_count | profile | weak_attribute | start_at | end_at |
|----|-----------|------------|---------|----------------|----------|--------|
| uuid-1 | 古戦場 | 65 | unite_fight | 1 | 2025-10-15 19:00:00 | 2025-10-22 23:59:00 |
| uuid-2 | ゼノ撃滅戦 | 30 | xeno_clash | 2 | 2025-10-20 17:00:00 | 2025-10-27 16:59:00 |

## 関連テーブル

### 関連テーブル

- **master.event_schedule_details**: `profile` で論理的に紐づけ（外部キー制約なし）

## タイムスタンプ自動更新

このテーブルは SeaORM の `ActiveModelBehavior` を使用して、以下のタイムスタンプが自動設定されます:

- **created_at**: レコード作成時に自動設定
- **updated_at**: レコード作成時・更新時に自動設定

詳細は [sea_orm_timestamp_automation.md](../../design/database/sea_orm_timestamp_automation.md) を参照してください。

## Rust実装

- **エンティティファイル**: `src/models/entities/master/event_schedules.rs`
- **マイグレーションファイル**: `migration/src/m*_create_event_schedules.rs`
- **実装状況**: ✅ 実装済み

### エンティティ定義（抜粋）

```rust
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(schema_name = "master", table_name = "event_schedules")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub event_type: String,
    pub event_count: i64,
    pub profile: String,
    pub weak_attribute: i32,
    pub start_at: DateTime,  // タイムゾーンなし
    pub end_at: DateTime,    // タイムゾーンなし
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}
```

## 備考

- **カラム名変更**: `row_id` → `id` に変更
- **NOT NULL変更**: `profile`, `weak_attribute` が NULLABLE から NOT NULL に変更されました
- **外部キー削除**: `weak_attribute` の外部キー制約は削除されました（論理的には elements.id を参照）
- **タイムゾーン**: `start_at`, `end_at` はタイムゾーンなし型（TIMESTAMP）を使用。これはスプレッドシート（JST）と一致させるためです
- 主キーは UUID で、`auto_increment = false` として定義されています
- `event_type` と `event_count` の組み合わせで一意に識別
- `profile` は `event_schedule_details` との紐づけに使用
- `weak_attribute` でイベントの有利属性を管理（外部キー制約はないが、論理的には elements テーブルを参照）
