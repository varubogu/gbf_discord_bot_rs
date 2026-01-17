# 通知スケジュール（notifications）

## 概要

**テーブル物理名**: `notifications`
**スキーマ名**: `worker`
**テーブルタイプ**: Transaction
**テーブルスコープ**: All（全ギルド共通）
**実装状況**: ✅ 実装済み

## 用途

実際に通知を送信するスケジュールを管理します。イベントスケジュールから具体的な日時に展開された通知で、通知処理の実行対象となります。

## カラム定義

| カラム名 | 型 | 制約 | 説明 |
|---------|-----|------|------|
| id | SERIAL | PK, NOT NULL | 通知ID（主キー、自動採番） |
| task_id | INT | NOT NULL, UNIQUE, FK | スケジュールタスクID（scheduled_tasks.idを参照） |
| guild_id | BIGINT | NOT NULL | 対象ギルドID（Discord Guild ID） |
| channel_id | BIGINT | NOT NULL | 通知先チャンネルID（Discord Channel ID） |
| message_text_id | TEXT | NOT NULL | 送信メッセージテンプレートID（message_texts.idを参照） |
| is_sent | BOOLEAN | NOT NULL, DEFAULT false | 送信済みフラグ |
| created_at | TIMESTAMPTZ | NOT NULL | 作成日時（UTC） |
| updated_at | TIMESTAMPTZ | NOT NULL | 更新日時（UTC） |

## 制約

### プライマリキー
- `id`

### 外部キー
実装では外部キー制約は定義されていません（相互参照の問題を避けるため）

### UNIQUE制約
なし

### NOT NULL制約
- `id`, `task_id`, `guild_id`, `channel_id`, `message_text_id`, `is_sent`, `created_at`, `updated_at`

## インデックス

- **プライマリキーインデックス**: `id`（自動作成）
- **外部キーインデックス**: `task_id`（scheduled_tasksとのJOIN用）
- **推奨追加インデックス**: `guild_id`（検索性能向上のため）

**注:**
- `schedule_datetime`は`scheduled_tasks`テーブルに存在するため、`notifications`テーブルには不要
- 通知の実行日時は`scheduled_tasks.schedule_datetime`を参照する

## データサンプル

| id | task_id | guild_id | channel_id | message_text_id | is_sent |
|----|---------|----------|-----------|----------------|---------|
| 1 | 100 | 123456789 | 987654321 | DAILY_MISSION | false |
| 2 | 101 | 123456789 | 987654321 | BORDER_UPDATE | true |

**注:** `schedule_datetime`は`scheduled_tasks`テーブル（task_id=100, 101）を参照

## 関連テーブル

### 参照先テーブル（論理的な関連）

- **master.message_texts**: `message_text_id` で参照（多対1）
- **guild_master.guilds**: `guild_id` で参照（多対1）

### 参照元テーブル

- **worker.notification_rel_event_schedules**: 通知とイベントスケジュールの関連（1対多）
- **worker.notification_rel_battle_recruitments**: 通知とマルチバトル募集の関連（1対多）

## タイムスタンプ自動更新

このテーブルは SeaORM の `ActiveModelBehavior` を使用して、以下のタイムスタンプが自動設定されます:

- **created_at**: レコード作成時に自動設定
- **updated_at**: レコード作成時・更新時に自動設定

詳細は [sea_orm_timestamp_automation.md](../../design/database/sea_orm_timestamp_automation.md) を参照してください。

## Rust実装

- **エンティティファイル**: `src/models/entities/worker/notifications.rs`
- **マイグレーションファイル**: `migration/src/m*_create_notifications.rs`
- **実装状況**: ✅ 実装済み

### エンティティ定義（抜粋）

```rust
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(schema_name = "worker", table_name = "notifications")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub task_id: i32,
    pub guild_id: i64,
    pub channel_id: i64,
    pub message_text_id: String,
    pub is_sent: bool,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}
```

**注:**
- `schedule_datetime`は`scheduled_tasks`テーブルに存在するため、`notifications`テーブルには不要
- 通知の実行日時は`scheduled_tasks.schedule_datetime`を参照する

## 備考

- **テーブル名変更**: 以前の設計では `schedules` という名称でしたが、実装では `notifications` に変更されました
- **設計変更**: 以前の設計から大幅に変更されました:
  - 主キー: UUID (`row_id`) → SERIAL (`id`)
  - 削除されたカラム: `parent_schedule_id`, `parent_schedule_detail_id`（別テーブル `notification_rel_*` に分離）
  - カラム名変更: `message_id` → `message_text_id`（メッセージテンプレートIDを参照）
  - 追加されたカラム: `is_sent`（送信済みフラグ）

- **通知状態管理**:
  - `is_sent`: 通知が送信済みかどうか（false=未送信、true=送信済み）
  - デフォルト値は false

- **イベントスケジュールとの関連**:
  - イベントスケジュールとの関連は `notification_rel_event_schedules` テーブルで管理
  - マルチバトル募集との関連は `notification_rel_battle_recruitments` テーブルで管理

- **通知バッチ処理**:
  - `scheduled_tasks.schedule_datetime` を基準に定期的に実行
  - `is_sent = false` のレコードが処理対象
  - 送信完了後、`is_sent = true` に更新

- **外部キー制約なし**:
  - 相互参照の問題を避けるため、外部キー制約は定義されていません
  - 必要に応じて手動でJOINクエリを実装

## データクリーンアップ

古い通知データは定期的にクリーンアップされます。詳細は [data_cleanup_system.md](../../design/features/data_cleanup_system.md) を参照してください。
