# 最終処理実行日時（last_process_times）

## 概要

**テーブル物理名**: `last_process_times`
**スキーマ名**: `worker`
**テーブルタイプ**: History
**テーブルスコープ**: All（全ギルド共通）
**実装状況**: ✅ 実装済み

## 用途

バッチ処理の最終実行時刻を記録します。定期実行処理の重複防止や、障害復旧時の基準時刻として使用されます。

## カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|-----|------|------|
| process_type | INTEGER | PK, NOT NULL | 処理種類ID（1=スケジュール、2=スプレッドシート読込、3=スプレッドシート書込、4=マルチ募集スケジュール） |
| execute_time | TIMESTAMPTZ | NULLABLE | 処理実行日時（UTC） |
| memo | TEXT | NOT NULL | メモ（処理の説明） |
| created_at | TIMESTAMPTZ | NOT NULL | 作成日時（UTC） |
| updated_at | TIMESTAMPTZ | NOT NULL | 更新日時（UTC） |

## 制約

### プライマリキー
- `process_type`

### 外部キー
なし

### UNIQUE制約
なし

### NOT NULL制約
- `process_type`, `memo`, `created_at`, `updated_at`

## インデックス

- **プライマリキーインデックス**: `process_type`（自動作成）

## データサンプル

| process_type | execute_time | memo |
|-------------|-------------|------|
| 1 | 2025-10-23 12:00:00+00 | 最終スケジュール実行日時 |
| 2 | 2025-10-23 11:55:00+00 | 最終Googleスプレッドシート読み込み日時 |
| 3 | 2025-10-23 00:00:00+00 | 最終Googleスプレッドシート書き込み日時 |
| 4 | 2025-10-23 10:00:00+00 | 最終マルチ募集スケジュール実行日時 |

## タイムスタンプ自動更新

このテーブルは SeaORM の `ActiveModelBehavior` を使用して、以下のタイムスタンプが自動設定されます:

- **created_at**: レコード作成時に自動設定
- **updated_at**: レコード作成時・更新時に自動設定

詳細は [sea_orm_timestamp_automation.md](../../design/database/sea_orm_timestamp_automation.md) を参照してください。

## Rust実装

- **エンティティファイル**: `src/models/entities/worker/last_process_times.rs`
- **マイグレーションファイル**: `migration/src/m*_create_last_process_times.rs`
- **実装状況**: ✅ 実装済み

### エンティティ定義（抜粋）

```rust
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(schema_name = "worker", table_name = "last_process_times")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub process_type: i32,
    pub execute_time: Option<DateTimeUtc>,
    pub memo: String,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

// 処理種類のenum定義
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LastProcessType {
    Schedule = 1,
    SpreadsheetLoad = 2,
    SpreadsheetPush = 3,
    BattleRecruitmentSchedule = 4,
}
```

## 備考

- **スキーマ変更**: このテーブルは `worker` スキーマに配置されています（`master` ではありません）
- **NOT NULL変更**: `memo` が NULLABLE から NOT NULL に変更されました
- グローバルな定期処理の実行時刻を記録
- `execute_time` が NULL の場合は未実行を示す
- バッチ処理の重複実行を防止するために使用
- 障害復旧時に最後の正常実行時刻を確認可能
- `LastProcessType` enum で処理種類を型安全に管理可能
