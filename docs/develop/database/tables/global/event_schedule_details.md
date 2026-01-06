# イベント詳細スケジュール（event_schedule_details）

## 概要

**テーブル物理名**: `event_schedule_details`
**スキーマ名**: `master`
**テーブルタイプ**: Reference
**テーブルスコープ**: All（全ギルド共通）
**実装状況**: ✅ 実装済み

## 用途

イベント期間内の詳細スケジュール（デイリーミッション、ボーダー更新など）を定義します。相対日時とprofileでイベントスケジュールと紐づけ、具体的な通知日時に展開されます。

## カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|-----|------|------|
| id | UUID | PK, NOT NULL | 行ID（プライマリキー、auto_increment = false） |
| profile | TEXT | NOT NULL | イベントスケジュールとの紐づけプロファイル |
| start_day_relative | TEXT | NOT NULL | 開始日からの相対日（例: "1", "1-5", "final"） |
| time | TEXT | NOT NULL | イベント時間（例: "23:59", "05:00"） |
| schedule_name | TEXT | NOT NULL | スケジュール名（例: デイリーミッション、ボーダー更新） |
| message_text_id | TEXT | NOT NULL | 通知メッセージID（message_texts.idを参照） |
| notification_channel_type | INTEGER | NOT NULL | 通知先チャンネル種類（channel_types.idを参照） |
| reactions | TEXT | NOT NULL | 通知メッセージに付与するリアクション（絵文字） |
| created_at | TIMESTAMPTZ | NOT NULL | 作成日時（UTC） |
| updated_at | TIMESTAMPTZ | NOT NULL | 更新日時（UTC） |

## 制約

### プライマリキー
- `id`（auto_increment = false）

### 外部キー
なし（論理的には `message_text_id` と `notification_channel_type` が参照関係あり）

### UNIQUE制約
なし

### NOT NULL制約
- `id`, `profile`, `start_day_relative`, `time`, `schedule_name`, `message_text_id`, `notification_channel_type`, `reactions`, `created_at`, `updated_at`

## インデックス

- **プライマリキーインデックス**: `id`（自動作成）

## データサンプル

| id | profile | start_day_relative | time | schedule_name | message_text_id | notification_channel_type | reactions |
|----|---------|-------------------|------|--------------|----------------|--------------------------|-----------|
| uuid-1 | unite_fight | 1 | 05:00 | デイリーミッション | DAILY_MISSION | 2 | ✅ |
| uuid-2 | unite_fight | 1-5 | 23:59 | ボーダー更新 | BORDER_UPDATE | 2 | 📊 |
| uuid-3 | xeno_clash | 1 | 00:00 | イベント開始 | EVENT_START | 2 | 🎉 |

## 関連テーブル

### 関連テーブル

- **master.event_schedules**: `profile` で論理的に紐づけ（外部キー制約なし）
- **master.message_texts**: `message_text_id` で論理的に参照（外部キー制約なし）
- **master.channel_types**: `notification_channel_type` で論理的に参照（外部キー制約なし）

## タイムスタンプ自動更新

このテーブルは SeaORM の `ActiveModelBehavior` を使用して、以下のタイムスタンプが自動設定されます:

- **created_at**: レコード作成時に自動設定
- **updated_at**: レコード作成時・更新時に自動設定

詳細は [sea_orm_timestamp_automation.md](../../design/database/sea_orm_timestamp_automation.md) を参照してください。

## Rust実装

- **エンティティファイル**: `src/models/entities/master/event_schedule_details.rs`
- **マイグレーションファイル**: `migration/src/m*_create_event_schedule_details.rs`
- **実装状況**: ✅ 実装済み

### エンティティ定義（抜粋）

```rust
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(schema_name = "master", table_name = "event_schedule_details")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub profile: String,
    pub start_day_relative: String,
    pub time: String,
    pub schedule_name: String,
    pub message_text_id: String,
    pub notification_channel_type: i32,
    pub reactions: String,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}
```

## 備考

- **カラム名変更**: `row_id` → `id`, `message_id` → `message_text_id` に変更
- **カラム削除**: `guild_id`, `channel_id` が削除され、代わりに `notification_channel_type` が追加されました
- **設計変更**: 個別の guild_id/channel_id 指定から、channel_type による通知先指定に変更されました
- **NOT NULL変更**: `message_text_id`, `notification_channel_type`, `reactions` が NULLABLE から NOT NULL に変更されました
- 主キーは UUID で、`auto_increment = false` として定義されています
- `profile` で event_schedules と紐づけ
- `start_day_relative` は相対日を示し、"1"は初日、"1-5"は1日目から5日目まで、"final"は最終日
- `time` は時刻を示し、"HH:MM"形式
- `notification_channel_type` で通知先のチャンネル種類を指定（全ギルドの該当チャンネルに通知）
